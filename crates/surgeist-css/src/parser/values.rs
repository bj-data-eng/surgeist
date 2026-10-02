use cssparser::{ParseError, Parser, Token};

use crate::error::{CssFeatureId, Error, basic, unsupported_value_at};
use crate::syntax::*;
use crate::validation::parse_global_keyword;

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
    CssFeatureId::new("interop.value.light-dark-color"),
    CssFeatureId::new("interop.value.contrast-color"),
    CssFeatureId::new("interop.value.device-cmyk"),
];

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

// Ordinary tokens are collected with their exact spelling and origin. Math roots
// use the owning dimensional context without a float scalar reconstruction.
macro_rules! checked_length_parser {
    ($name:ident, $owner:ident, $calculation:ident, $root:ident) => {
        pub(super) fn $name<'i, 't>(
            input: &mut Parser<'i, 't>,
            numeric: &NumericInputContext<'_>,
            context: &str,
        ) -> Result<crate::$owner, ParseError<'i, Error>> {
            input.skip_whitespace();
            let state = input.state();
            let location = input.current_source_location();
            let root_offset = input.position().byte_index();
            let value = match input.next().map_err(basic)? {
                Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
                    input.reset(&state);
                    let component = numeric.collect(input).map_err(|error| {
                        unsupported_value_at(
                            numeric.error_location(&error, location, root_offset),
                            None,
                            format!("invalid {context}"),
                        )
                    })?;
                    crate::$owner::try_from_component(component)
                }
                Token::Function(name) if is_math_function(name) => {
                    let expression =
                        parse_numeric_function(input, &state, numeric, CalculationRoot::$root)?;
                    crate::$owner::try_from_calculation(crate::$calculation::from_expression(
                        expression,
                    ))
                }
                Token::Ident(ident) => {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        format!("unsupported {context} `{ident}`"),
                    ))
                }
                Token::Function(name) => {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        format!("unsupported length function `{name}` for {context}"),
                    ))
                }
                token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
            };
            value.map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    format!("invalid {context} length domain"),
                )
            })
        }
    };
}
checked_length_parser!(
    parse_length,
    CssSpecifiedLength,
    CssLengthCalculation,
    Length
);
checked_length_parser!(
    parse_nonnegative_length,
    CssSpecifiedNonNegativeLength,
    CssLengthCalculation,
    Length
);
checked_length_parser!(
    parse_length_percentage,
    CssSpecifiedLengthPercentage,
    CssLengthPercentageCalculation,
    LengthPercentage
);
checked_length_parser!(
    parse_nonnegative_length_percentage,
    CssSpecifiedNonNegativeLengthPercentage,
    CssLengthPercentageCalculation,
    LengthPercentage
);

// Scalar consumers share exact ordinary admission and typed symbolic roots.
macro_rules! checked_scalar_parser {
    ($name:ident, $owner:ident, $calculation:ident, $root:ident, $token:ident) => {
        pub(super) fn $name<'i, 't>(
            input: &mut Parser<'i, 't>,
            numeric: &NumericInputContext<'_>,
            context: &str,
        ) -> Result<crate::$owner, ParseError<'i, Error>> {
            input.skip_whitespace();
            let state = input.state();
            let location = input.current_source_location();
            let root_offset = input.position().byte_index();
            let value = match input.next().map_err(basic)? {
                Token::$token { .. } => {
                    input.reset(&state);
                    let component = numeric.collect(input).map_err(|error| {
                        unsupported_value_at(
                            numeric.error_location(&error, location, root_offset),
                            None,
                            format!("invalid {context}"),
                        )
                    })?;
                    crate::$owner::try_from_component(component)
                }
                Token::Function(name) if is_math_function(name) => {
                    let expression =
                        parse_numeric_function(input, &state, numeric, CalculationRoot::$root)?;
                    crate::$owner::try_from_calculation(crate::$calculation::from_expression(
                        expression,
                    ))
                }
                token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
            };
            value.map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    format!("invalid {context} nonnegative domain"),
                )
            })
        }
    };
}
checked_scalar_parser!(
    parse_nonnegative_number,
    CssSpecifiedNonNegativeNumber,
    CssNumberCalculation,
    Number,
    Number
);
checked_scalar_parser!(
    parse_nonnegative_percentage,
    CssSpecifiedNonNegativePercentage,
    CssPercentageCalculation,
    Percentage,
    Percentage
);

pub(super) fn parse_shadow_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssSpecifiedLength, ParseError<'i, Error>> {
    parse_length(input, numeric, "shadow offset")
}
pub(super) fn parse_shadow_nonnegative_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssSpecifiedNonNegativeLength, ParseError<'i, Error>> {
    parse_nonnegative_length(input, numeric, "nonnegative shadow length")
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
            parse_integer_literal(input, numeric).map(CssIntegerValue::Literal)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .map(CssIntegerValue::Calculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_integer_literal<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssIntegerLiteral, ParseError<'i, Error>> {
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
    crate::CssIntegerLiteral::try_from_component(component)
        .map_err(|_| unsupported_value_at(location, None, "value must have integer token syntax"))
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

// Admission is shared; calling properties retain their own grammar and diagnostics.
macro_rules! specified_scalar_parser {
    ($name:ident, $token:ident, $owner:ident, $calculation:ident, $root:ident) => {
        pub(super) fn $name<'i, 't>(
            input: &mut Parser<'i, 't>,
            numeric: &crate::numeric::NumericInputContext<'_>,
            context: &str,
        ) -> Result<$owner, ParseError<'i, Error>> {
            input.skip_whitespace();
            let start = input.state();
            let location = input.current_source_location();
            let root_offset = input.position().byte_index();
            let value = match input.next().map_err(basic)? {
                Token::$token { .. } => {
                    input.reset(&start);
                    let component = numeric.collect(input).map_err(|_| {
                        unsupported_value_at(
                            location,
                            None,
                            format!("invalid {context} numeric literal"),
                        )
                    })?;
                    $owner::try_from_component(component)
                }
                Token::Function(name) if crate::numeric::is_math_function(name) => {
                    let expression =
                        parse_numeric_function(input, &start, numeric, CalculationRoot::$root)?;
                    $owner::try_from_calculation($calculation::from_expression(expression))
                }
                token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
            };
            value.map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    format!(
                        "{context} requires a {}",
                        stringify!($root).to_ascii_lowercase()
                    ),
                )
            })
        }
    };
}
specified_scalar_parser!(
    parse_specified_number,
    Number,
    CssSpecifiedNumber,
    CssNumberCalculation,
    Number
);
specified_scalar_parser!(
    parse_specified_percentage,
    Percentage,
    CssSpecifiedPercentage,
    CssPercentageCalculation,
    Percentage
);

pub(super) fn parse_specified_number_literal<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    context: &str,
) -> Result<CssSpecifiedNumber, ParseError<'i, Error>> {
    let start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {}
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
    input.reset(&start);
    parse_specified_number(input, numeric, context)
}

#[derive(Clone, Copy)]
pub(super) enum AngleParserContext {
    Transform,
    Filter,
    Gradient,
    ImageOrientation,
    ShapeRotation,
}
impl AngleParserContext {
    fn label(self) -> &'static str {
        match self {
            Self::Transform => "transform",
            Self::Filter => "filter",
            Self::Gradient => "gradient",
            Self::ImageOrientation => "image-orientation",
            Self::ShapeRotation => "shape rotation",
        }
    }
}

pub(super) fn parse_angle_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    context: AngleParserContext,
) -> Result<crate::CssAngleValue, ParseError<'i, Error>> {
    let (component, location, offset) = collect_angle_component(input, numeric, context)?;
    crate::CssAngleValue::from_parser_component(component, numeric)
        .map_err(|error| angle_error(numeric, &error, location, offset, context))
}

pub(super) fn parse_angle_or_zero<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    context: AngleParserContext,
) -> Result<crate::CssAngleOrZero, ParseError<'i, Error>> {
    let (component, location, offset) = collect_angle_component(input, numeric, context)?;
    crate::CssAngleOrZero::from_parser_component(component, numeric)
        .map_err(|error| angle_error(numeric, &error, location, offset, context))
}

fn collect_angle_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    context: AngleParserContext,
) -> Result<(crate::CssComponentValue, cssparser::SourceLocation, usize), ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = numeric
        .collect(input)
        .map_err(|error| angle_error(numeric, &error, location, offset, context))?;
    Ok((component, location, offset))
}

fn angle_error<'i>(
    numeric: &NumericInputContext<'_>,
    error: &crate::CssNumericConstructionError,
    fallback: cssparser::SourceLocation,
    offset: usize,
    context: AngleParserContext,
) -> ParseError<'i, Error> {
    let mut location = numeric.error_location(error, fallback, offset);
    // Map the exact recovered angle closure through the original component map;
    // canonical output coordinates never become an authored source position.
    if let Some(origin @ crate::CssValueOrigin::ImplicitClosure { .. }) = error.origin()
        && let NumericInputContext::Components(_, serialized) = numeric
        && let Some(segment) = serialized.segments().iter().find(|segment| matches!(segment.origin(), crate::CssSerializedOrigin::Token(candidate) if candidate == origin))
    {
        let mut input = cssparser::ParserInput::new(&serialized.as_css()[..segment.byte_range().start]);
        let mut parser = cssparser::Parser::new(&mut input);
        while parser.next_including_whitespace_and_comments().is_ok() {}
        location = parser.current_source_location();
    }
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::invalid_component_value(location, component.clone());
    }
    unsupported_value_at(location, None, format!("invalid {} angle", context.label()))
}
