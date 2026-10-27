//! Generic numeric values, calculations, units and their root and range markers.

pub(crate) mod angle;
pub(crate) mod aspect_ratio;
#[cfg(test)]
pub(crate) mod borrowed_admission_tests;
pub(crate) mod bounded_decimal;
pub(crate) mod bounded_expression;
pub(crate) mod color_operands;
pub(crate) mod controlled_check;
pub(crate) mod expression;
pub(crate) mod integer_value;
pub(crate) mod ordinary_expression;
pub(crate) mod projection;
pub(crate) mod specified_numeric;
pub(crate) mod syntax;
pub(crate) mod units;
pub(crate) mod value;

#[doc(inline)]
pub use angle::{CssAngleOrZero, CssZeroLiteral};
#[doc(inline)]
pub use aspect_ratio::CssSpecifiedRatio;
#[doc(inline)]
pub use expression::{
    CssCalculationConstantRef, CssCalculationExpressionRef, CssCalculationFunctionRef,
    CssCalculationProductFactorRef, CssCalculationProductOperator, CssCalculationProductRef,
    CssCalculationProfileChannelRef, CssCalculationSizeRef, CssCalculationSumOperator,
    CssCalculationSumRef, CssCalculationSumTermRef, CssCalculationTreeCountingRef,
    CssCalculationType, CssCalculationUnaryRef, CssCalculationValueRef, CssCalculationVariableRef,
    CssMathFunction, CssNumericConstant, CssNumericConstructionError, CssNumericDimension,
    CssNumericLiteralRef, CssNumericType, CssNumericUnit, CssRoundingStrategy,
    CssTreeCountingFunction,
};
#[doc(inline)]
pub use syntax::{
    CssAngleUnit, CssFiniteNumber, CssFrequencyUnit, CssLengthUnit, CssRatio, CssResolutionUnit,
    CssTimeUnit,
};
#[doc(inline)]
pub use value::{
    CssAngle, CssAnglePercentage, CssAnyRange, CssCalculation, CssDimensionRoot, CssFlex,
    CssFrequency, CssFrequencyPercentage, CssHintedNumber, CssInteger, CssLength,
    CssLengthPercentage, CssNonNegative, CssNumber, CssNumeric, CssNumericLiteral, CssNumericRoot,
    CssPercentage, CssPositive, CssRange, CssResolution, CssTime,
};
