//! Generic numeric values, calculations, units and their root and range markers.
//!
//! [`CssNumeric`] is the one generic value for a number, dimension or percentage of a root
//! marker (length, angle, time, ...) with an optional range marker. It holds either an exact
//! [`CssNumericLiteral`] or a [`CssCalculation`], and checked construction rejects values outside
//! the root's domain or range. Nothing is resolved against layout, timelines or device context.
//!
//! # Typed authored calculations
//!
//! Calculation roots preserve authored numeric values and units without resolving layout,
//! timelines, or device context. Literal construction is checked, and every expression is
//! inspected through borrowed views while the owned compound representation remains private.
//!
//! ```
//! use surgeist_css::numeric::{CssAngle, CssAngleUnit, CssCalculation, CssCalculationExpressionRef, CssCalculationType, CssCalculationValueRef};
//!
//! let angle = CssCalculation::<CssAngle>::try_literal("-0.5", CssAngleUnit::Turns)
//!     .expect("checked authored angle");
//! assert_eq!(angle.result_type(), CssCalculationType::Angle);
//! assert!(matches!(
//!     angle.expression(),
//!     CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(value))
//!         if value.representation() == "-0.5" && value.unit() == Some("turn")
//! ));
//! ```
//!
//! Calculation trees remain authored and symbolic. This crate checks finite literal values and
//! dimensional validity, but it does not resolve relative units, evaluate computed ranges, or
//! run animation timelines.

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
