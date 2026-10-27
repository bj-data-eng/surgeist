use super::property_input::{Parser, ParserState, Site};
use crate::diagnostic::error::property::ParseError;
use cssparser::Token;

use crate::diagnostic::CssFeatureId;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::numeric::expression::CssCalculationExpression;
use crate::numeric::{
    CssCalculation, CssHintedNumber, CssInteger, CssLength, CssLengthPercentage, CssNonNegative,
    CssNumber, CssNumeric, CssNumericLiteral, CssPercentage, CssPositive,
};
#[cfg(test)]
use crate::numeric::{
    CssCalculationExpressionRef, CssCalculationProductOperator, CssCalculationSumOperator,
    CssCalculationType, CssCalculationValueRef,
};
use crate::values::common::{CssCustomIdent, CssIdent};

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
) -> std::result::Result<f64, cssparser::ParseError<'i, crate::diagnostic::CssError>> {
    // cssparser exposes percentages after dividing by 100 and rounding to f32.
    // Re-multiplication can change the authored magnitude (30% -> 30.000002).
    // The caller passes exactly the consumed percentage token, without trivia.
    let value = token_css
        .strip_suffix('%')
        .and_then(|numeric| numeric.parse::<f64>().ok())
        .filter(|value| value.is_finite());
    value.ok_or_else(|| crate::diagnostic::error::unexpected_at(location))
}

/// The ordinary tokens a Values 4 numeric type collects as a literal.
///
/// Other tokens are reported as unexpected; a math function enters the type's
/// calculation root instead (Values 4 §10: math is valid wherever its result
/// type is).
#[derive(Clone, Copy)]
pub(super) enum LiteralTokens {
    Number,
    Percentage,
    Dimension,
    /// Number, percentage and dimension tokens.
    Numeric,
}

impl LiteralTokens {
    fn admit(self, token: &Token<'_>) -> bool {
        match self {
            Self::Number => matches!(token, Token::Number { .. }),
            Self::Percentage => matches!(token, Token::Percentage { .. }),
            Self::Dimension => matches!(token, Token::Dimension { .. }),
            Self::Numeric => matches!(
                token,
                Token::Number { .. } | Token::Percentage { .. } | Token::Dimension { .. }
            ),
        }
    }
}

type NumericResult<T> = Result<T, crate::numeric::CssNumericConstructionError>;

/// A Values 4 numeric type: its calculation root, its literal tokens and the
/// owner's checked literal and calculation constructors. The literal
/// constructor receives the property-scoped numeric context, which alone
/// grants quirky-length permission.
pub(super) struct NumericType<T> {
    pub(super) root: CalculationRoot,
    pub(super) tokens: LiteralTokens,
    pub(super) literal:
        fn(crate::component::CssComponentValue, &NumericInputContext<'_>) -> NumericResult<T>,
    pub(super) calculation: fn(CssCalculationExpression) -> NumericResult<T>,
}

/// The one parser for Values 4 numeric types and their ranges.
///
/// Ordinary tokens are collected with their exact spelling and origin; math
/// roots use the type's calculation root without a float scalar
/// reconstruction. Every failure is reported at the value's first significant
/// token, or at the failing descendant a construction error's path names.
pub(super) fn parse_numeric_type<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    ty: &NumericType<T>,
) -> Result<T, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position();
    let value = match input.next().map_err(basic)? {
        token if ty.tokens.admit(token) => {
            input.reset(&state);
            input
                .collect_numeric_component(numeric)
                .and_then(|component| (ty.literal)(component, numeric))
        }
        Token::Function(name) if is_math_function(name) => {
            let expression = parse_numeric_function(input, &state, numeric, ty.root)?;
            (ty.calculation)(expression)
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    value.map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, root_offset))
    })
}

macro_rules! numeric_types {
    ($(
        $(#[$meta:meta])*
        $descriptor:ident, $parser:ident: $owner:ty, $root:ident, $tokens:ident, $literal:expr;
    )*) => {$(
        $(#[$meta])*
        pub(super) const $descriptor: NumericType<$owner> = NumericType {
            root: CalculationRoot::$root,
            tokens: LiteralTokens::$tokens,
            literal: $literal,
            calculation: |expression| {
                <$owner>::from_parser_calculation(CssCalculation::from_expression(expression))
            },
        };
        pub(super) fn $parser<'i, 't>(
            input: &mut Parser<'i, 't>,
            numeric: &NumericInputContext<'_>,
        ) -> Result<$owner, ParseError<'i, Error>> {
            parse_numeric_type(input, numeric, &$descriptor)
        }
    )*};
}

numeric_types! {
    /// `<length>`, admitting quirky lengths where the property grants them.
    LENGTH, parse_length: CssNumeric<CssLength>, Length, Numeric,
        CssNumeric::from_property_component;
    /// `<length [0,∞]>`, admitting quirky lengths where the property grants them.
    NONNEGATIVE_LENGTH, parse_nonnegative_length: CssNumeric<CssLength, CssNonNegative>,
        Length, Numeric, CssNumeric::from_property_component;
    /// `<length-percentage>`, admitting quirky lengths where the property grants them.
    LENGTH_PERCENTAGE, parse_length_percentage: CssNumeric<CssLengthPercentage>,
        LengthPercentage, Numeric, CssNumeric::from_property_component;
    /// `<length-percentage [0,∞]>`, admitting quirky lengths where the property grants them.
    NONNEGATIVE_LENGTH_PERCENTAGE, parse_nonnegative_length_percentage:
        CssNumeric<CssLengthPercentage, CssNonNegative>, LengthPercentage, Numeric,
        CssNumeric::from_property_component;
    /// `<number>`.
    NUMBER, parse_specified_number: CssNumeric<CssNumber>, Number, Number,
        |component, _| CssNumeric::try_from_component(component);
    /// `<number [0,∞]>`.
    NONNEGATIVE_NUMBER, parse_nonnegative_number: CssNumeric<CssNumber, CssNonNegative>,
        Number, Number, |component, _| CssNumeric::try_from_component(component);
    /// `<percentage>`.
    PERCENTAGE, parse_specified_percentage: CssNumeric<CssPercentage>, Percentage, Percentage,
        |component, _| CssNumeric::try_from_component(component);
    /// `<percentage [0,∞]>`.
    NONNEGATIVE_PERCENTAGE, parse_nonnegative_percentage:
        CssNumeric<CssPercentage, CssNonNegative>, Percentage, Percentage,
        |component, _| CssNumeric::try_from_component(component);
}

// Scalar consumers share exact ordinary admission and typed symbolic roots.
// Percentage-permitting consumers retain Number results with a dimensional hint
// separately from the pure scalar wrappers and their ordinary range checks.
pub(super) fn parse_hinted_number_calculation<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssCalculation<CssHintedNumber>, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Function(name) if is_math_function(name) => {
            parse_numeric_function(input, &start, numeric, CalculationRoot::HintedNumber)
                .map(CssCalculation::<CssHintedNumber>::from_expression)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_shadow_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::numeric::CssNumeric<crate::numeric::CssLength>, ParseError<'i, Error>> {
    parse_length(input, numeric)
}
pub(super) fn parse_shadow_nonnegative_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<
    crate::numeric::CssNumeric<crate::numeric::CssLength, crate::numeric::CssNonNegative>,
    ParseError<'i, Error>,
> {
    parse_nonnegative_length(input, numeric)
}

use crate::numeric::expression::NumericInputContext;
use crate::numeric::expression::{CalculationRoot, is_math_function};

pub(super) fn parse_numeric_function<'i, 't>(
    input: &mut Parser<'i, 't>,
    before_opener: &ParserState<'i>,
    numeric: &NumericInputContext<'_>,
    root: CalculationRoot,
) -> Result<CssCalculationExpression, ParseError<'i, Error>> {
    input.reset(before_opener);
    input.skip_whitespace();
    let root_offset = input.position();
    let location = input.current_source_location();
    let component = input.collect_component().map_err(|error| {
        calculation_error(
            root_offset
                .source_origin(error.origin())
                .unwrap_or(location),
        )
    })?;
    let values = crate::numeric::expression::CalculationComponents::single(component)
        .map_err(|_| calculation_error(location))?;
    numeric.admit(values, root).map_err(|error| {
        calculation_error(numeric.property_error_location(&error, location, root_offset))
    })
}

pub(super) fn calculation_error<'i>(location: Site<'i>) -> ParseError<'i, Error> {
    unexpected_at(location)
}

/// Shared exact integer admission for authored number tokens and integer-root math.
pub(super) fn parse_integer_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNumeric<CssInteger>, ParseError<'i, Error>> {
    /// `<integer>`; math keeps its authored Integer root without range checks.
    const INTEGER: NumericType<CssNumeric<CssInteger>> = NumericType {
        root: CalculationRoot::Integer,
        tokens: LiteralTokens::Number,
        literal: |component, _| {
            CssNumericLiteral::from_component_token(component)
                .map(CssNumeric::from_literal)
                .map_err(crate::numeric::CssNumericConstructionError::component)
        },
        calculation: |expression| {
            CssNumeric::from_parser_calculation(CssCalculation::from_expression(expression))
        },
    };
    parse_numeric_type(input, numeric, &INTEGER)
}

pub(super) fn parse_integer_literal<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::numeric::CssNumericLiteral<crate::numeric::CssInteger>, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position();
    let component = input.collect_numeric_component(numeric).map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, offset))
    })?;
    crate::numeric::CssNumericLiteral::<crate::numeric::CssInteger>::try_from_component(component)
        .map_err(|_| unexpected_at(location))
}

pub(super) fn parse_custom_ident_from_str_at<'i>(
    ident: &str,
    location: Site<'i>,
) -> std::result::Result<CssCustomIdent, ParseError<'i, Error>> {
    CssCustomIdent::try_from_ident(CssIdent::new(ident)).ok_or_else(|| unexpected_at(location))
}

/// Native name grammars retain their own source position while sharing the
/// exact custom-identifier construction owner with property input.
pub(super) fn parse_custom_ident_from_str_at_source<'i>(
    ident: &str,
    location: cssparser::SourceLocation,
) -> Result<CssCustomIdent, cssparser::ParseError<'i, crate::diagnostic::CssError>> {
    CssCustomIdent::try_from_ident(CssIdent::new(ident))
        .ok_or_else(|| crate::diagnostic::error::unexpected_at(location))
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

/// Ordinary resolution reuses the exact nonnegative literal and symbolic Resolution owners.
pub(super) fn parse_ordinary_resolution<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<
    crate::numeric::CssNumeric<crate::numeric::CssResolution, crate::numeric::CssNonNegative>,
    ParseError<'i, Error>,
> {
    // The numeric context's admission applies its recovery policy to math.
    const RESOLUTION: NumericType<
        crate::numeric::CssNumeric<crate::numeric::CssResolution, crate::numeric::CssNonNegative>,
    > = NumericType {
        root: CalculationRoot::Resolution,
        tokens: LiteralTokens::Dimension,
        literal: |component, _| {
            CssNumericLiteral::from_component_token(component)
                .map_err(crate::numeric::CssNumericConstructionError::component)
                .and_then(CssNumeric::try_from_literal)
        },
        calculation: |expression| {
            crate::numeric::CssNumeric::<
                crate::numeric::CssResolution,
                crate::numeric::CssNonNegative,
            >::from_parser_calculation(crate::numeric::CssCalculation::<
                crate::numeric::CssResolution,
            >::from_expression(expression))
        },
    };
    parse_numeric_type(input, numeric, &RESOLUTION)
}

pub(super) fn parse_angle_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::numeric::CssNumeric<crate::numeric::CssAngle>, ParseError<'i, Error>> {
    let (component, location, offset) = collect_angle_component(input, numeric)?;
    crate::numeric::CssNumeric::<crate::numeric::CssAngle>::from_parser_component(
        component, numeric,
    )
    .map_err(|error| angle_error(numeric, &error, location, offset))
}

pub(super) fn parse_angle_or_zero<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::numeric::CssAngleOrZero, ParseError<'i, Error>> {
    let (component, location, offset) = collect_angle_component(input, numeric)?;
    crate::numeric::CssAngleOrZero::from_parser_component(component, numeric)
        .map_err(|error| angle_error(numeric, &error, location, offset))
}

fn collect_angle_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<(crate::component::CssComponentValue, Site<'i>, Site<'i>), ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| angle_error(numeric, &error, location, offset))?;
    Ok((component, location, offset))
}

pub(super) fn angle_error<'i>(
    numeric: &NumericInputContext<'_>,
    error: &crate::numeric::CssNumericConstructionError,
    fallback: Site<'i>,
    offset: Site<'i>,
) -> ParseError<'i, Error> {
    let location = numeric.property_error_location(error, fallback, offset);
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

pub(super) fn parse_positive_integer_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssNumeric<CssInteger, CssPositive>, ParseError<'i, Error>> {
    /// `<integer [1,∞]>`; math keeps its authored Integer root.
    const POSITIVE_INTEGER: NumericType<CssNumeric<CssInteger, CssPositive>> = NumericType {
        root: CalculationRoot::Integer,
        tokens: LiteralTokens::Number,
        literal: |component, _| {
            CssNumericLiteral::from_component_token(component)
                .map_err(crate::numeric::CssNumericConstructionError::component)
                .and_then(CssNumeric::try_from_literal)
        },
        calculation: |expression| {
            CssNumeric::from_parser_calculation(CssCalculation::from_expression(expression))
        },
    };
    parse_numeric_type(input, numeric, &POSITIVE_INTEGER)
}

#[cfg(test)]
mod typed_calculation_tests {
    use super::*;
    use crate::diagnostic::CssErrorCode;

    fn parse(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::diagnostic::CssError> {
        parse_complete(&format!("calc({source})"), root)
    }

    fn parse_complete(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::diagnostic::CssError> {
        let snapshot = crate::source::CssSourceSnapshot::new(source);
        let numeric = NumericInputContext::parsed(&snapshot);
        let document =
            crate::syntax_consumption::source_document(source, &snapshot, 0).map_err(|error| {
                let crate::component::CssValueOrigin::Parsed(origin) = error.origin() else {
                    unreachable!("source normalization errors retain their original source");
                };
                let position = origin.span().start();
                crate::diagnostic::error::from_parse_error(
                    source,
                    crate::diagnostic::error::unexpected_at(cssparser::SourceLocation {
                        line: position.line().value(),
                        column: position.column().value().saturating_add(1),
                    }),
                )
            })?;
        let mut parser = Parser::new(&document);
        let before = parser.state();
        parser
            .parse_entirely(|input| parse_numeric_function(input, &before, &numeric, root))
            .map_err(|error| error.into_source_error())
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
