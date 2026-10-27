//! One generic Values 4 numeric value over sealed root and range markers.
//!
//! Values 4 §5–7 make the root type select a numeric domain, and §10.12 makes
//! `[min,max]` a range annotation on that same type. [`CssNumeric`] retains an
//! exact ordinary [`CssNumericLiteral`] or checked root math
//! ([`CssCalculation`]); its [`CssRange`] marker is tied to its root, so a
//! range of another root cannot be named.

use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;

use super::{
    CssCalculationExpressionRef, CssCalculationType, CssNumericConstructionError, CssNumericType,
    CssNumericUnit,
};
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssNumericTokenKind, CssNumericTokenRef, CssSerializedValue,
    CssValueOrigin, CssValueTokenRef,
};
use crate::numeric::expression::CssNumericConstructionErrorKind as Kind;
use crate::numeric::expression::{CalculationRoot, CssCalculationExpression, NumericInputContext};
use crate::numeric::specified_numeric::{CalculationAdmission, admit_calculation};
use crate::numeric::{
    CssAngleUnit, CssFrequencyUnit, CssLengthUnit, CssResolutionUnit, CssTimeUnit,
};
use crate::serialization::exact_decimal::LexicalDecimal;
use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::serialization::{CssSerializationError, CssSerializationLimits};
use crate::source::CssSourcePosition;

type Result<T> = std::result::Result<T, CssNumericConstructionError>;
type SerializationResult<T> = std::result::Result<T, CssSerializationError>;

/// The closed set of numeric roots, read from a root marker's sealed identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootKind {
    Number,
    HintedNumber,
    Integer,
    Percentage,
    Length,
    LengthPercentage,
    Flex,
    Angle,
    AnglePercentage,
    Time,
    Frequency,
    FrequencyPercentage,
    Resolution,
}

impl RootKind {
    pub(crate) const fn calculation_root(self) -> CalculationRoot {
        match self {
            Self::Number => CalculationRoot::Number,
            Self::HintedNumber => CalculationRoot::HintedNumber,
            Self::Integer => CalculationRoot::Integer,
            Self::Percentage => CalculationRoot::Percentage,
            Self::Length => CalculationRoot::Length,
            Self::LengthPercentage => CalculationRoot::LengthPercentage,
            Self::Flex => CalculationRoot::Flex,
            Self::Angle => CalculationRoot::Angle,
            Self::AnglePercentage => CalculationRoot::AnglePercentage,
            Self::Time => CalculationRoot::Time,
            Self::Frequency => CalculationRoot::Frequency,
            Self::FrequencyPercentage => CalculationRoot::FrequencyPercentage,
            Self::Resolution => CalculationRoot::Resolution,
        }
    }

    /// Whether the contextual quirky Number-as-px interpretation can apply.
    const fn admits_quirky_lengths(self) -> bool {
        matches!(self, Self::Length | Self::LengthPercentage)
    }

    /// Whether specified serialization captures math as scratch text before
    /// appending it, rather than streaming it into the enclosing output.
    const fn captures_math(self) -> bool {
        matches!(
            self,
            Self::Number
                | Self::HintedNumber
                | Self::Percentage
                | Self::Length
                | Self::LengthPercentage
                | Self::Flex
        )
    }

    /// Whether a strict cursor rejects recovered closure before math admission.
    const fn rejects_closure_first(self) -> bool {
        matches!(self, Self::Angle | Self::Time)
    }
}

pub(crate) const fn root_kind<T: CssNumericRoot>() -> RootKind {
    <T as crate::sealed::NumericRoot>::ROOT.0
}

/// A Values 4 numeric data type that can be a value or calculation root.
///
/// The implementors are the uninhabited root markers of this crate, such as
/// [`CssLength`] and [`CssAngle`]; no other crate can implement it.
pub trait CssNumericRoot: crate::sealed::NumericRoot + Copy + Eq + fmt::Debug + 'static {}

/// A root whose ordinary literal always carries a unit of one vocabulary.
///
/// Lengths are not a dimension root, because the exact unitless zero is an
/// ordinary length literal.
pub trait CssDimensionRoot: CssNumericRoot {
    /// The unit vocabulary of this root's literals.
    type Unit: Copy + Eq + fmt::Debug + crate::sealed::DimensionUnit;
}

/// A Values 4 range annotation valid for root `T`.
///
/// Each range is implemented only for the roots it constrains, so a range of
/// another root cannot be named. It is sealed: only CSS defines ranges.
///
/// ```compile_fail,E0277
/// use surgeist_css::numeric::{CssNumeric, CssTime};
/// use surgeist_css::values::font::CssObliqueAngleRange;
/// fn mismatched(_: CssNumeric<CssTime, CssObliqueAngleRange>) {}
/// ```
///
/// ```compile_fail,E0277
/// use surgeist_css::numeric::{CssNumber, CssRange};
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// enum Positive {}
/// impl CssRange<CssNumber> for Positive {}
/// ```
pub trait CssRange<T: CssNumericRoot>:
    crate::sealed::RangeBounds<T> + Copy + Eq + fmt::Debug + 'static
{
}

macro_rules! roots {
    ($($(#[$meta:meta])* $name:ident => $kind:ident;)*) => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {}
        impl crate::sealed::NumericRoot for $name {
            const ROOT: crate::sealed::Root = crate::sealed::Root(RootKind::$kind);
        }
        impl CssNumericRoot for $name {}
    )*};
}

roots! {
    /// The `<number>` root.
    CssNumber => Number;
    /// A Number result retaining a dimensional percentage hint.
    ///
    /// Only percentage-permitting consumers admit this root. The hint names a
    /// dimension without resolving its percentage basis; it has no ordinary
    /// literal, and pure Number roots remain a separate checked domain.
    CssHintedNumber => HintedNumber;
    /// The `<integer>` root, without a machine-integer magnitude bound.
    CssInteger => Integer;
    /// The `<percentage>` root.
    CssPercentage => Percentage;
    /// The `<length>` root, including the exact unitless zero.
    CssLength => Length;
    /// The `<length-percentage>` root.
    CssLengthPercentage => LengthPercentage;
    /// The `<flex>` root of Grid track sizing.
    CssFlex => Flex;
    /// The `<angle>` root.
    CssAngle => Angle;
    /// The `<angle-percentage>` root, with an unresolved percentage basis.
    CssAnglePercentage => AnglePercentage;
    /// The `<time>` root.
    CssTime => Time;
    /// The `<frequency>` root.
    CssFrequency => Frequency;
    /// The `<frequency-percentage>` root, with an unresolved percentage basis.
    CssFrequencyPercentage => FrequencyPercentage;
    /// The `<resolution>` root.
    CssResolution => Resolution;
}

macro_rules! dimension_roots {
    ($($root:ident => $unit:ident, $variant:ident;)*) => {$(
        impl CssDimensionRoot for $root {
            type Unit = $unit;
        }
        impl crate::sealed::DimensionUnit for $unit {
            fn from_numeric(unit: CssNumericUnit) -> Option<Self> {
                match unit {
                    CssNumericUnit::$variant(unit) => Some(unit),
                    _ => None,
                }
            }
            fn into_numeric(self) -> CssNumericUnit {
                CssNumericUnit::$variant(self)
            }
        }
    )*};
}

dimension_roots! {
    CssAngle => CssAngleUnit, Angle;
    CssTime => CssTimeUnit, Time;
    CssFrequency => CssFrequencyUnit, Frequency;
    CssResolution => CssResolutionUnit, Resolution;
}

/// The unrestricted range `[-∞,∞]`, valid for every root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssAnyRange {}
impl<T: CssNumericRoot> crate::sealed::RangeBounds<T> for CssAnyRange {
    fn admits(_: &CssNumericLiteral<T>) -> bool {
        true
    }
}
impl<T: CssNumericRoot> CssRange<T> for CssAnyRange {}

/// The range `[0,∞]`, valid for every root except [`CssHintedNumber`].
///
/// An exact negative literal is rejected, including one too small for any
/// machine float; signed zero is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssNonNegative {}

macro_rules! nonnegative {
    ($($root:ident),* $(,)?) => {$(
        impl crate::sealed::RangeBounds<$root> for CssNonNegative {
            fn admits(literal: &CssNumericLiteral<$root>) -> bool {
                !LexicalDecimal::new(literal.numeric().representation()).negative
            }
        }
        impl CssRange<$root> for CssNonNegative {}
    )*};
}

nonnegative!(
    CssNumber,
    CssInteger,
    CssPercentage,
    CssLength,
    CssLengthPercentage,
    CssFlex,
    CssAngle,
    CssAnglePercentage,
    CssTime,
    CssFrequency,
    CssFrequencyPercentage,
    CssResolution,
);

/// The integer range `[1,∞]`, valid only for [`CssInteger`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssPositive {}
impl crate::sealed::RangeBounds<CssInteger> for CssPositive {
    fn admits(literal: &CssNumericLiteral<CssInteger>) -> bool {
        let decimal = LexicalDecimal::new(literal.numeric().representation());
        !decimal.negative && !decimal.is_zero()
    }
}
impl CssRange<CssInteger> for CssPositive {}

/// An exact ordinary numeric token of root `T`.
///
/// The original spelling, unit and provenance are retained without binary64
/// conversion.
///
/// # Equality
///
/// Two literals are equal when they have the same unit and the same exact
/// value (CSS Typed OM 1 "equal numeric values"). Source origin and the
/// spelling of the value are ignored, so `2`, `2.0` and `+2` are equal, as are
/// `2x` and `2dppx` (one unit under two spellings), while `96px` and `1in` are
/// not. A length's unitless zero and quirky Number have
/// unit px, as Typed OM reifies them.
#[derive(Clone, Debug)]
pub struct CssNumericLiteral<T: CssNumericRoot> {
    component: Box<CssComponentValue>,
    unit: Option<CssNumericUnit>,
    // A Number token admitted by quirky-length context denotes a px length.
    quirky: bool,
    root: PhantomData<fn() -> T>,
}

/// The unit of a reified numeric literal.
#[derive(Clone, Copy, PartialEq)]
enum ReifiedUnit {
    Number,
    Percent,
    Unit(CssNumericUnit),
}

impl<T: CssNumericRoot> PartialEq for CssNumericLiteral<T> {
    fn eq(&self, other: &Self) -> bool {
        self.reified_unit() == other.reified_unit()
            && LexicalDecimal::new(self.numeric().representation())
                .value_eq(&LexicalDecimal::new(other.numeric().representation()))
    }
}

impl<T: CssNumericRoot> Eq for CssNumericLiteral<T> {}

impl<T: CssNumericRoot> CssNumericLiteral<T> {
    /// Checks one ordinary token against root `T` without binary64 conversion.
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] when the token is not an ordinary
    /// literal of this root.
    pub fn try_from_component(component: CssComponentValue) -> Result<Self> {
        let unit = literal_unit(root_kind::<T>(), &component)?;
        Ok(Self::retain(component, unit, false))
    }

    /// Checks one ordinary token, reporting a mismatch as an invalid token.
    pub(crate) fn from_component_token(
        component: CssComponentValue,
    ) -> std::result::Result<Self, CssComponentValueError> {
        match literal_unit(root_kind::<T>(), &component) {
            Ok(unit) => Ok(Self::retain(component, unit, false)),
            Err(_) => Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            )),
        }
    }

    /// The quirky-length interpretation of a nonzero Number token.
    fn quirky(component: CssComponentValue) -> Self {
        Self::retain(component, None, true)
    }

    fn retain(component: CssComponentValue, unit: Option<CssNumericUnit>, quirky: bool) -> Self {
        Self {
            component: Box::new(component),
            unit,
            quirky,
            root: PhantomData,
        }
    }

    /// Returns the exact authored numeric token.
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        literal_number(&self.component)
    }

    /// Borrows the original checked component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Borrows the original component provenance.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    /// Whether both literals have the same decoded unit and interpretation.
    pub(crate) fn structural_kind_eq(&self, other: &Self) -> bool {
        self.unit == other.unit && self.quirky == other.quirky
    }

    /// The unit of the reified value (CSS Typed OM 1 §5 "reify a numeric
    /// value"): a length's unitless zero and quirky Number reify as px.
    fn reified_unit(&self) -> ReifiedUnit {
        match (self.unit, self.component.view()) {
            (Some(unit), _) => ReifiedUnit::Unit(unit),
            (None, CssComponentValueRef::Token(CssValueTokenRef::Percentage(_))) => {
                ReifiedUnit::Percent
            }
            (None, _)
                if matches!(
                    root_kind::<T>(),
                    RootKind::Length | RootKind::LengthPercentage
                ) =>
            {
                ReifiedUnit::Unit(CssNumericUnit::Length(CssLengthUnit::Px))
            }
            (None, _) => ReifiedUnit::Number,
        }
    }

    pub(crate) const fn is_quirky(&self) -> bool {
        self.quirky
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        let captured = self.capture_specified(context)?;
        context.append(output, &captured)
    }

    /// Charges one node and returns this literal's canonical specified text.
    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        let kind = root_kind::<T>();
        match kind {
            RootKind::Length | RootKind::LengthPercentage if self.quirky => {
                crate::numeric::specified_numeric::capture_quirky_length(&self.component, context)
            }
            RootKind::Number
            | RootKind::HintedNumber
            | RootKind::Percentage
            | RootKind::Length
            | RootKind::LengthPercentage => {
                crate::numeric::specified_numeric::capture_literal(&self.component, context)
            }
            RootKind::Flex => {
                context.charge_nodes(1)?;
                if context.output_suppressed() {
                    return Ok(String::new());
                }
                let limit = context
                    .remaining_bytes()
                    .checked_sub(2)
                    .ok_or_else(byte_limit)?;
                let mut output = crate::serialization::numeric_formatting::format_css_number(
                    self.numeric().representation(),
                    0,
                    limit,
                )?;
                context.append_temporary(&mut output, "fr")?;
                Ok(output)
            }
            RootKind::Integer => {
                context.charge_nodes(1)?;
                if context.output_suppressed() {
                    return Ok(String::new());
                }
                crate::numeric::integer_value::serialize_integer_digits(
                    self.numeric().representation(),
                    context.remaining_bytes(),
                )
            }
            RootKind::Resolution => self.capture_resolution(context),
            RootKind::Time => {
                let unit = self.unit.expect("checked time unit");
                let shift =
                    i128::from(super::units::decimal_shift(unit).expect("decimal time ratio"));
                self.capture_coefficient(
                    context,
                    shift,
                    super::units::canonical_suffix(super::CssNumericDimension::Time)
                        .expect("canonical time unit"),
                )
            }
            RootKind::Angle
            | RootKind::AnglePercentage
            | RootKind::Frequency
            | RootKind::FrequencyPercentage => {
                let suffix = self.unit.map_or("%", super::units::suffix);
                self.capture_coefficient(context, 0, suffix)
            }
        }
    }

    fn capture_coefficient(
        &self,
        context: &mut SpecifiedSerializationContext,
        shift: i128,
        suffix: &str,
    ) -> SerializationResult<String> {
        context.charge_nodes(1)?;
        if context.output_suppressed() {
            return Ok(String::new());
        }
        crate::serialization::specified_serialization::format_coefficient(
            self.numeric().representation(),
            shift,
            suffix,
            context.remaining_bytes(),
        )
    }

    // CSSOM selects dots per CSS pixel; conversion uses the exact unit ratio.
    fn capture_resolution(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        context.charge_nodes(1)?;
        if context.output_suppressed() {
            return Ok(String::new());
        }
        let unit = self.unit.expect("checked resolution unit");
        if unit == CssNumericUnit::Resolution(CssResolutionUnit::Dppx) {
            return crate::serialization::specified_serialization::format_coefficient(
                self.numeric().representation(),
                0,
                "dppx",
                context.remaining_bytes(),
            );
        }
        let number_bytes = context
            .remaining_bytes()
            .checked_sub(4)
            .ok_or_else(byte_limit)?;
        let factor = super::units::exact_factor(unit)
            .expect("Values 4 fixes an exact ratio for every resolution unit");
        let mut number = crate::serialization::exact_decimal::ExactRational::format_generic_number(
            self.numeric().representation(),
            factor,
            number_bytes,
            context,
        )?;
        number.try_reserve(4).map_err(|_| {
            CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
        })?;
        number.push_str("dppx");
        Ok(number)
    }
}

impl<T: CssDimensionRoot> CssNumericLiteral<T> {
    /// Constructs an exact coefficient with the selected unit.
    ///
    /// # Errors
    ///
    /// Returns [`CssComponentValueError`] when `number` is not one exact CSS
    /// number spelling for this unit.
    pub fn try_new(
        number: &str,
        unit: T::Unit,
    ) -> std::result::Result<Self, CssComponentValueError> {
        use crate::sealed::DimensionUnit as _;
        let unit = unit.into_numeric();
        let component = CssComponentValue::try_dimension(number, super::units::suffix(unit))?;
        if dimension_unit(&component) != Some(unit) {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ));
        }
        Ok(Self::retain(component, Some(unit), false))
    }

    /// Returns the decoded unit without converting the coefficient.
    pub fn unit(&self) -> T::Unit {
        use crate::sealed::DimensionUnit as _;
        self.unit
            .and_then(T::Unit::from_numeric)
            .expect("checked dimension unit")
    }
}

impl CssNumericLiteral<CssInteger> {
    /// Returns the exact value when its magnitude fits `i32`.
    ///
    /// Authored integers are not machine-bounded; a larger magnitude returns
    /// `None` rather than a clamped value.
    #[must_use]
    pub fn value(&self) -> Option<i32> {
        crate::numeric::integer_value::exact_i32(self.numeric().representation())
    }

    /// Constructs the canonical programmatic spelling of a machine integer.
    #[must_use]
    pub fn from_i32(value: i32) -> Self {
        Self::try_from_component(
            CssComponentValue::try_number(&value.to_string()).expect("valid integer spelling"),
        )
        .expect("checked integer token")
    }

    /// Whether this lexical integer is mathematically zero, regardless of sign.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        LexicalDecimal::new(self.numeric().representation()).is_zero()
    }

    /// Whether this lexical integer is mathematically negative; signed zero is not negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        LexicalDecimal::new(self.numeric().representation()).negative
    }

    /// Compares mathematical integer values without changing authored identity.
    ///
    /// Explicit plus signs, leading zeroes and provenance do not affect this
    /// ordering. Negative values precede zero and positive values; signed zeroes
    /// compare equally; `==` agrees with this ordering for integer literals.
    #[must_use]
    pub fn compare_value(&self, other: &Self) -> Ordering {
        fn significant_digits(value: &CssNumericLiteral<CssInteger>) -> &str {
            let text = value.numeric().representation();
            text.strip_prefix(['+', '-'])
                .unwrap_or(text)
                .trim_start_matches('0')
        }
        let left = significant_digits(self);
        let right = significant_digits(other);
        let left_negative = self.numeric().representation().starts_with('-') && !left.is_empty();
        let right_negative = other.numeric().representation().starts_with('-') && !right.is_empty();
        match (left_negative, right_negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                let magnitude = left.len().cmp(&right.len()).then_with(|| left.cmp(right));
                if left_negative {
                    magnitude.reverse()
                } else {
                    magnitude
                }
            }
        }
    }
}

impl CssNumericLiteral<CssAngle> {
    pub(crate) fn is_default_gradient_direction(&self) -> bool {
        let decimal = LexicalDecimal::new(self.numeric().representation());
        let (digits, exponent) = match self.unit() {
            CssAngleUnit::Degrees => ("18", 1),
            CssAngleUnit::Gradians => ("2", 2),
            CssAngleUnit::Turns => ("5", -1),
            CssAngleUnit::Radians => return false,
        };
        decimal.is_positive(digits, exponent)
    }
}

impl CssNumericLiteral<CssTime> {
    /// Exact ordinary duration equivalence for specified shorthand omission.
    /// Coefficients and origins remain authored; only standard s/ms conversion
    /// participates, without floating-point rounding or exponent expansion.
    fn equivalent(&self, other: &Self) -> bool {
        fn seconds(literal: &CssNumericLiteral<CssTime>) -> LexicalDecimal<'_> {
            LexicalDecimal::new(literal.numeric().representation()).shifted(
                super::units::decimal_shift(CssNumericUnit::Time(literal.unit()))
                    .expect("decimal time ratio"),
            )
        }
        seconds(self).value_eq(&seconds(other))
    }
}

fn byte_limit() -> CssSerializationError {
    CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::ByteLimit)
}

fn dimension_unit(component: &CssComponentValue) -> Option<CssNumericUnit> {
    match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) => {
            super::units::parse(unit)
        }
        _ => None,
    }
}

fn literal_number(component: &CssComponentValue) -> CssNumericTokenRef<'_> {
    match component.view() {
        CssComponentValueRef::Token(
            CssValueTokenRef::Number(number)
            | CssValueTokenRef::Percentage(number)
            | CssValueTokenRef::Dimension { number, .. },
        ) => number,
        _ => unreachable!("checked numeric literal"),
    }
}

/// Checks one ordinary token against a root and returns its decoded unit.
fn literal_unit(kind: RootKind, component: &CssComponentValue) -> Result<Option<CssNumericUnit>> {
    let mismatch = || CssNumericConstructionError::at(Kind::RootDomainMismatch, Some(component));
    let CssComponentValueRef::Token(token) = component.view() else {
        return Err(mismatch());
    };
    let dimension = |accepts: fn(CssNumericUnit) -> bool| match token {
        CssValueTokenRef::Dimension { .. } => dimension_unit(component)
            .filter(|unit| accepts(*unit))
            .map(Some)
            .ok_or_else(mismatch),
        _ => Err(mismatch()),
    };
    let percentage = matches!(token, CssValueTokenRef::Percentage(_));
    match kind {
        RootKind::Number => matches!(token, CssValueTokenRef::Number(_))
            .then_some(None)
            .ok_or_else(mismatch),
        RootKind::HintedNumber => Err(mismatch()),
        RootKind::Integer => matches!(
            token,
            CssValueTokenRef::Number(number) if number.kind() == CssNumericTokenKind::Integer
        )
        .then_some(None)
        .ok_or_else(mismatch),
        RootKind::Percentage => percentage.then_some(None).ok_or_else(mismatch),
        RootKind::Length | RootKind::LengthPercentage => match token {
            CssValueTokenRef::Number(number)
                if LexicalDecimal::new(number.representation()).is_zero() =>
            {
                Ok(None)
            }
            CssValueTokenRef::Percentage(_) if kind == RootKind::LengthPercentage => Ok(None),
            CssValueTokenRef::Dimension { unit, .. } => CssLengthUnit::from_css_unit(unit)
                .map(|unit| Some(CssNumericUnit::Length(unit)))
                .ok_or_else(mismatch),
            _ => Err(mismatch()),
        },
        RootKind::Flex => match token {
            CssValueTokenRef::Dimension { unit, .. } if unit.eq_ignore_ascii_case("fr") => {
                Ok(Some(CssNumericUnit::Flex))
            }
            _ => Err(mismatch()),
        },
        RootKind::Angle => dimension(|unit| matches!(unit, CssNumericUnit::Angle(_))),
        RootKind::AnglePercentage if percentage => Ok(None),
        RootKind::AnglePercentage => dimension(|unit| matches!(unit, CssNumericUnit::Angle(_))),
        RootKind::Time => dimension(|unit| matches!(unit, CssNumericUnit::Time(_))),
        RootKind::Frequency => dimension(|unit| matches!(unit, CssNumericUnit::Frequency(_))),
        RootKind::FrequencyPercentage if percentage => Ok(None),
        RootKind::FrequencyPercentage => {
            dimension(|unit| matches!(unit, CssNumericUnit::Frequency(_)))
        }
        RootKind::Resolution => dimension(|unit| matches!(unit, CssNumericUnit::Resolution(_))),
    }
}

/// A checked calculation whose result type is root `T`.
///
/// It retains the original component graph and its provenance; nothing is
/// evaluated, clamped or rounded.
///
/// A calculation of another root is a different type:
///
/// ```compile_fail,E0308
/// use surgeist_css::numeric::{CssAngle, CssCalculation, CssLength, CssNonNegative, CssNumeric};
/// fn mismatched(angle: CssCalculation<CssAngle>) {
///     let _ = CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(angle);
/// }
/// ```
///
/// ```compile_fail,E0308
/// use surgeist_css::numeric::{CssCalculation, CssHintedNumber, CssNumber, CssNumeric};
/// fn pure(value: CssCalculation<CssHintedNumber>) {
///     let _ = CssNumeric::<CssNumber>::try_from_calculation(value);
/// }
/// ```
///
/// # Equality
///
/// Two calculations are equal when they have the same operations and
/// functions with pairwise equal operands in the same order (CSS Typed OM 1
/// "equal numeric values"). Operands compare as literals do: by exact value
/// and by unit after alias resolution, so `calc(2x)` equals `calc(2dppx)`.
/// Source origin and spelling are ignored, so `calc(1px + 2px)` equals
/// `CALC(1.0px + 2px)` but not `calc(2px + 1px)`.
///
/// Redundant parentheses and nested `calc()` functions leave no node (Values
/// 4 "parse a calculation") and are seen through, so `calc((1px + 2px) + 3px)`
/// equals `calc(calc(1px + 2px) + 3px)`; grouping that changes the operator
/// tree does not, so `calc((1px + 2px) * 2)` differs from `calc(1px + 2px * 2)`.
/// Every other math function is a node: `min()` differs from `max()`. The root
/// function is not unwrapped, because Typed OM reifies a root `calc()` as a sum
/// around its argument: `calc(min(1px, 2px))` differs from `min(1px, 2px)`, as
/// `calc(1px)` differs from the literal `1px`.
#[derive(Clone, Debug)]
pub struct CssCalculation<T: CssNumericRoot> {
    pub(crate) expression: Box<CssCalculationExpression>,
    root: PhantomData<fn() -> T>,
}

impl<T: CssNumericRoot> PartialEq for CssCalculation<T> {
    fn eq(&self, other: &Self) -> bool {
        self.expression.value_eq(&other.expression)
    }
}

impl<T: CssNumericRoot> Eq for CssCalculation<T> {}

impl<T: CssNumericRoot> CssCalculation<T> {
    /// Checks closed components whose single significant root has result type `T`.
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] for recovered, malformed,
    /// mistyped or over-limit components.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self> {
        crate::numeric::expression::construct(values, root_kind::<T>().calculation_root(), limits)
            .map(Self::from_expression)
    }

    pub(crate) fn from_expression(expression: CssCalculationExpression) -> Self {
        Self {
            expression: Box::new(expression),
            root: PhantomData,
        }
    }

    /// Borrows the checked expression tree.
    pub fn expression(&self) -> CssCalculationExpressionRef<'_> {
        self.expression.as_ref().as_ref()
    }

    /// Returns the classified result type.
    pub fn result_type(&self) -> CssCalculationType {
        self.expression.result_type()
    }

    /// Returns the complete numeric type, including any percentage hint.
    pub fn numeric_type(&self) -> CssNumericType {
        self.expression.ty
    }

    /// Borrows the original component graph.
    pub fn components(&self) -> &CssComponentValues {
        self.expression
            .components
            .as_ref()
            .expect("checked root component graph")
    }

    /// Returns the parsed root's start position, when parsed.
    pub fn position(&self) -> Option<CssSourcePosition> {
        match &self.expression.origin {
            CssValueOrigin::Parsed(origin) => Some(origin.span().start()),
            _ => None,
        }
    }

    /// Borrows the root's parsed or programmatic provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        &self.expression.origin
    }

    /// Serializes the retained calculation as canonical tokens with their
    /// origins. `limits.max_css_bytes()` caps the output; the other
    /// component limits do not apply.
    ///
    /// # Errors
    ///
    /// Returns [`CssComponentValueError`] when the output exceeds the byte
    /// limit; no partial output is returned.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> std::result::Result<CssSerializedValue, CssComponentValueError> {
        let mut tokens = Vec::new();
        self.expression.canonical_tokens(&mut tokens);
        let value = CssSerializedValue::from_numeric_tokens(tokens)?;
        if value.as_css().len() > limits.max_css_bytes() {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::ByteLimit,
                self.origin().clone(),
            ));
        }
        Ok(value)
    }

    /// Streams the specified projection into the enclosing output.
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        crate::numeric::projection::project_specified_into(&self.expression, context, output)
            .map(|_| ())
    }

    /// Captures the specified projection as scratch text.
    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        crate::numeric::projection::capture_specified(&self.expression, context)
            .map(|(text, _)| text)
    }
}

impl<T: CssDimensionRoot> CssCalculation<T> {
    /// Constructs an exact ordinary dimension root, retaining the selected unit.
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] when `number` is not one exact
    /// CSS number spelling.
    pub fn try_literal(number: &str, unit: T::Unit) -> Result<Self> {
        let literal = CssNumericLiteral::<T>::try_new(number, unit)
            .map_err(CssNumericConstructionError::component)?;
        let values = CssComponentValues::try_new(vec![*literal.component], Default::default())
            .map_err(CssNumericConstructionError::component)?;
        Self::try_from_components(values, Default::default())
    }
}

fn programmatic_number(value: f32) -> Option<CssComponentValues> {
    if !value.is_finite() {
        return None;
    }
    CssComponentValues::try_new(
        vec![CssComponentValue::try_number(&value.to_string()).ok()?],
        Default::default(),
    )
    .ok()
}

fn programmatic_dimension(value: f32, unit: &str) -> Option<CssComponentValues> {
    if !value.is_finite() {
        return None;
    }
    CssComponentValues::try_new(
        vec![CssComponentValue::try_dimension(&value.to_string(), unit).ok()?],
        Default::default(),
    )
    .ok()
}

impl CssCalculation<CssNumber> {
    /// Constructs a finite programmatic number root.
    pub fn try_literal(value: f32) -> Option<Self> {
        Self::try_from_components(programmatic_number(value)?, Default::default()).ok()
    }
}

impl CssCalculation<CssPercentage> {
    /// Constructs a finite programmatic percentage root.
    pub fn try_literal(value: f32) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        Self::try_from_components(
            CssComponentValues::try_new(
                vec![CssComponentValue::try_token(&format!("{value}%")).ok()?],
                Default::default(),
            )
            .ok()?,
            Default::default(),
        )
        .ok()
    }
}

impl CssCalculation<CssLength> {
    /// Constructs a finite programmatic length root.
    pub fn try_dimension(value: f32, unit: CssLengthUnit) -> Option<Self> {
        Self::try_from_components(
            programmatic_dimension(value, unit.as_str())?,
            Default::default(),
        )
        .ok()
    }
}

impl CssCalculation<CssLengthPercentage> {
    /// Constructs a finite programmatic length root.
    pub fn try_dimension(value: f32, unit: CssLengthUnit) -> Option<Self> {
        Self::try_from_components(
            programmatic_dimension(value, unit.as_str())?,
            Default::default(),
        )
        .ok()
    }

    /// Constructs a finite programmatic percentage root.
    pub fn try_percentage(value: f32) -> Option<Self> {
        Self::try_from_components(
            CssCalculation::<CssPercentage>::try_literal(value)?
                .components()
                .clone(),
            Default::default(),
        )
        .ok()
    }
}

impl CssCalculation<CssInteger> {
    /// Whether the root is a math function whose result needs integer rounding.
    pub fn requires_rounding(&self) -> bool {
        matches!(
            self.expression.kind,
            crate::numeric::expression::NodeKind::Function { .. }
        )
    }

    /// Constructs the canonical programmatic root of a machine integer.
    pub fn literal(value: i32) -> Self {
        Self::try_from_components(
            CssComponentValues::try_new(
                vec![CssComponentValue::try_number(&value.to_string()).expect("integer spelling")],
                Default::default(),
            )
            .expect("single component"),
            Default::default(),
        )
        .expect("integer literal")
    }
}

/// An exact ordinary literal or retained root math, range-checked when literal.
///
/// `T` selects the Values 4 root type and `R` its range annotation. Ordinary
/// literals are checked against `R` exactly, without binary64 conversion;
/// math defers `R` to computed-value range handling (Values 4 §10.12).
///
/// # Equality
///
/// A literal never equals math. Literals and calculations compare as
/// [`CssNumericLiteral`] and [`CssCalculation`] do: by unit (after alias
/// resolution) and exact value, ignoring source origin and spelling. Within
/// math, redundant parentheses and nested `calc()` are seen through, while the
/// root function is not unwrapped.
///
/// # Specified serialization
///
/// Emits ordinary and finite calculated numbers with at most six fractional
/// places, nearest with ties away from zero. Times emit canonical seconds and
/// resolutions emit `dppx` after exact unit conversion; other dimensions keep
/// their canonical unit. Symbolic math is not resolved.
#[derive(Clone, Debug)]
pub struct CssNumeric<T: CssNumericRoot, R: CssRange<T> = CssAnyRange> {
    value: NumericValue<T>,
    range: PhantomData<fn() -> R>,
}

impl<T: CssNumericRoot, R: CssRange<T>> PartialEq for CssNumeric<T, R> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (NumericValue::Literal(left), NumericValue::Literal(right)) => left == right,
            (NumericValue::Calculation(left), NumericValue::Calculation(right)) => left == right,
            _ => false,
        }
    }
}

impl<T: CssNumericRoot, R: CssRange<T>> Eq for CssNumeric<T, R> {}

#[derive(Clone, Debug)]
enum NumericValue<T: CssNumericRoot> {
    Literal(CssNumericLiteral<T>),
    Calculation(CssCalculation<T>),
}

impl<T: CssNumericRoot, R: CssRange<T>> CssNumeric<T, R> {
    const fn retain(value: NumericValue<T>) -> Self {
        Self {
            value,
            range: PhantomData,
        }
    }

    /// Checks one ordinary token against `T` and `R` without binary64 conversion.
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] when the token is not a literal
    /// of root `T`, or lies outside `R`.
    pub fn try_from_component(component: CssComponentValue) -> Result<Self> {
        Self::try_from_literal(CssNumericLiteral::try_from_component(component)?)
    }

    /// Checks `R` on an exact literal.
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] when the literal lies outside `R`.
    pub fn try_from_literal(literal: CssNumericLiteral<T>) -> Result<Self> {
        if !<R as crate::sealed::RangeBounds<T>>::admits(&literal) {
            return Err(CssNumericConstructionError::at(
                Kind::OutOfRange,
                Some(literal.component()),
            ));
        }
        Ok(Self::retain(NumericValue::Literal(literal)))
    }

    /// Retains checked math; a bare numeric root re-enters literal admission.
    ///
    /// Components recovered by implicit closure are rejected. Math defers `R`
    /// to computed-value range handling (Values 4 §10.12).
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] for recovered components, and
    /// for a bare root that is not a literal of `T` within `R`.
    pub fn try_from_calculation(calculation: CssCalculation<T>) -> Result<Self> {
        Self::admit_calculation(calculation, CalculationAdmission::Checked)
    }

    /// Parser admission retains recovered syntax, which parsing reports separately.
    pub(crate) fn from_parser_calculation(calculation: CssCalculation<T>) -> Result<Self> {
        Self::admit_calculation(calculation, CalculationAdmission::Recovering)
    }

    fn admit_calculation(
        calculation: CssCalculation<T>,
        admission: CalculationAdmission,
    ) -> Result<Self> {
        admit_calculation(
            calculation,
            CssCalculation::components,
            admission,
            Self::try_from_component,
            |calculation| Self::retain(NumericValue::Calculation(calculation)),
        )
    }

    /// Admits one property component; quirky-length context alone admits a
    /// nonzero Number token as a px length.
    pub(crate) fn from_property_component(
        component: CssComponentValue,
        numeric: &NumericInputContext<'_>,
    ) -> Result<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            return Self::try_from_component(component);
        };
        // Ordinary exact zero keeps its ordinary serialization and identity.
        if !root_kind::<T>().admits_quirky_lengths()
            || !numeric.allows_quirky_lengths()
            || LexicalDecimal::new(number.representation()).is_zero()
        {
            return Self::try_from_component(component);
        }
        Self::try_from_literal(CssNumericLiteral::quirky(component))
    }

    /// Admits one parsed or checked dimension component.
    ///
    /// An ordinary token takes the literal path, where a token of another
    /// type is an invalid component. Otherwise the numeric context's
    /// admission policy applies; angles and times additionally report a
    /// strict context's recovered closure before math admission.
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &NumericInputContext<'_>,
    ) -> Result<Self> {
        if matches!(component.view(), CssComponentValueRef::Token(_)) {
            return CssNumericLiteral::from_component_token(component)
                .map_err(CssNumericConstructionError::component)
                .and_then(Self::try_from_literal);
        }
        let values = CssComponentValues::try_new(vec![component], Default::default())
            .map_err(CssNumericConstructionError::component)?;
        if root_kind::<T>().rejects_closure_first() {
            match context.ordinary() {
                NumericInputContext::QuirkyLengths(_) => {
                    unreachable!("ordinary numeric provenance")
                }
                NumericInputContext::Parsed(_) => {}
                NumericInputContext::Components(..) | NumericInputContext::Checked => {
                    reject_recovered(&values)?;
                }
            }
        }
        let expression = context.admit(values, root_kind::<T>().calculation_root())?;
        Self::from_parser_calculation(CssCalculation::from_expression(expression))
    }

    /// Borrows the exact ordinary literal, when present.
    pub fn literal(&self) -> Option<&CssNumericLiteral<T>> {
        match &self.value {
            NumericValue::Literal(literal) => Some(literal),
            NumericValue::Calculation(_) => None,
        }
    }

    /// Borrows the retained checked math, when present.
    pub fn calculation(&self) -> Option<&CssCalculation<T>> {
        match &self.value {
            NumericValue::Literal(_) => None,
            NumericValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            NumericValue::Literal(literal) => literal.origin(),
            NumericValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Relaxes the range annotation; total.
    pub fn into_any_range(self) -> CssNumeric<T, CssAnyRange> {
        CssNumeric::retain(self.value)
    }

    /// Serializes this value as canonical specified CSS.
    ///
    /// The text is context-independent: nothing is computed, resolved
    /// or substituted, and symbolic values stay symbolic. `limits`
    /// bounds input nodes, projection nodes and output UTF-8 bytes
    /// cumulatively across the value and all of its children.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] when a limit is exhausted or the value
    /// cannot be represented. No partial text is returned and the value is
    /// unchanged.
    pub fn serialize_specified(
        &self,
        limits: CssSerializationLimits,
    ) -> SerializationResult<String> {
        crate::serialization::specified_rule_serialization::serialize_specified(self, limits)
    }

    /// Narrows the range, rechecking a literal; retained math keeps its
    /// recovery provenance and defers the range.
    pub(crate) fn try_narrow<N: CssRange<T>>(self) -> Result<CssNumeric<T, N>> {
        match self.value {
            NumericValue::Literal(literal) => CssNumeric::try_from_literal(literal),
            NumericValue::Calculation(calculation) => {
                Ok(CssNumeric::retain(NumericValue::Calculation(calculation)))
            }
        }
    }

    /// Narrows the range under checked admission: recovered math is rejected.
    pub(crate) fn try_narrow_checked<N: CssRange<T>>(self) -> Result<CssNumeric<T, N>> {
        self.ensure_closed()?;
        self.try_narrow()
    }

    /// Borrows the original ordinary token, when this is a literal.
    pub(crate) fn literal_component(&self) -> Option<&CssComponentValue> {
        self.literal().map(CssNumericLiteral::component)
    }

    /// The first recovered closure in retained math.
    pub(crate) fn first_implicit_origin(&self) -> Option<&CssValueOrigin> {
        self.calculation()
            .and_then(|value| value.components().first_implicit_origin())
    }

    pub(crate) fn ensure_closed(&self) -> Result<()> {
        match self.calculation() {
            Some(value) => reject_recovered(value.components()),
            None => Ok(()),
        }
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            NumericValue::Literal(literal) => literal.capture_specified(context),
            NumericValue::Calculation(calculation) => calculation.capture_specified(context),
        }
    }

    /// Appends one value to a caller's cumulative specified-CSS budget.
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match &self.value {
            NumericValue::Literal(literal) => literal.append_specified(context, output),
            NumericValue::Calculation(calculation) if root_kind::<T>().captures_math() => {
                let captured = calculation.capture_specified(context)?;
                context.append(output, &captured)
            }
            NumericValue::Calculation(calculation) => calculation.append_specified(context, output),
        }
    }
}

impl<T: CssNumericRoot> CssNumeric<T> {
    /// Retains a literal under the unrestricted range, which admits every literal.
    pub(crate) fn from_literal(literal: CssNumericLiteral<T>) -> Self {
        Self::retain(NumericValue::Literal(literal))
    }
}

impl<R: CssRange<CssLength>> CssNumeric<CssLength, R> {
    /// Whether an original Number token has the contextual pixel interpretation.
    ///
    /// The literal still retains that unchanged authored Number token.
    pub fn is_quirky_length(&self) -> bool {
        self.literal().is_some_and(CssNumericLiteral::is_quirky)
    }
}

impl<R: CssRange<CssLengthPercentage>> CssNumeric<CssLengthPercentage, R> {
    /// Whether an original Number token has the contextual pixel interpretation.
    ///
    /// The literal still retains that unchanged authored Number token.
    pub fn is_quirky_length(&self) -> bool {
        self.literal().is_some_and(CssNumericLiteral::is_quirky)
    }
}

impl CssNumeric<CssLength> {
    /// Constructs the exact unitless-zero initial length.
    #[must_use]
    pub fn zero() -> Self {
        Self::try_from_component(CssComponentValue::try_number("0").expect("zero token"))
            .expect("zero length")
    }
}

impl<T: CssNumericRoot, R: CssRange<T>> CssNumeric<T, R> {
    /// The exact unitless zero of a length root whose range admits it.
    pub(crate) fn exact_zero() -> Self {
        Self::try_from_component(CssComponentValue::try_number("0").expect("zero token"))
            .expect("length root admitting zero")
    }

    /// Shortest-form literal equality, or `None` when either side is math.
    ///
    /// Values 4 Appendix C makes `<quirky-length>` a type of `<length>`; the
    /// retained Number denotes the px length with the same exact decimal, so it
    /// equals that explicit px literal. Other units stay distinct.
    pub(crate) fn inverse_literal_eq(&self, other: &Self) -> Option<bool> {
        let (NumericValue::Literal(left), NumericValue::Literal(right)) =
            (&self.value, &other.value)
        else {
            return None;
        };
        let quirky_px = |quirky: &CssComponentValue, explicit: &CssComponentValue| match (
            quirky.view(),
            explicit.view(),
        ) {
            (
                CssComponentValueRef::Token(CssValueTokenRef::Number(quirky)),
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }),
            ) => {
                unit.eq_ignore_ascii_case("px")
                    && LexicalDecimal::new(quirky.representation())
                        .value_eq(&LexicalDecimal::new(number.representation()))
            }
            _ => false,
        };
        Some(match (left.quirky, right.quirky) {
            (true, true) | (false, false) => {
                crate::numeric::specified_numeric::ordinary_length_literal_equal(
                    left.component(),
                    right.component(),
                )
            }
            (true, false) => quirky_px(left.component(), right.component()),
            (false, true) => quirky_px(right.component(), left.component()),
        })
    }
}

impl<R: CssRange<CssTime>> CssNumeric<CssTime, R> {
    /// Whether this is an ordinary literal whose exact value is zero; math,
    /// even math that evaluates to zero, is not.
    pub(crate) fn is_exact_ordinary_zero(&self) -> bool {
        self.literal().is_some_and(|literal| {
            LexicalDecimal::new(literal.numeric().representation()).is_zero()
        })
    }

    /// Specified shortening identity: literals compare after exact s/ms
    /// conversion, and math by its authored spelling, ignoring origin.
    pub(crate) fn specified_value_eq<O: CssRange<CssTime>>(
        &self,
        other: &CssNumeric<CssTime, O>,
    ) -> bool {
        match (&self.value, &other.value) {
            (NumericValue::Literal(left), NumericValue::Literal(right)) => left.equivalent(right),
            (NumericValue::Calculation(left), NumericValue::Calculation(right)) => {
                left.expression.structural_eq(&right.expression)
            }
            _ => false,
        }
    }

    /// Exact specified equality for the bounded inverse consumer. Ordinary
    /// unit equivalence remains borrowed; retained math schedules its real work.
    pub(crate) fn specified_inverse_eq<O: CssRange<CssTime>>(
        &self,
        other: &CssNumeric<CssTime, O>,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<bool> {
        match (&self.value, &other.value) {
            (NumericValue::Literal(left), NumericValue::Literal(right)) => {
                Ok(left.equivalent(right))
            }
            (NumericValue::Calculation(left), NumericValue::Calculation(right)) => left
                .expression
                .specified_inverse_eq(&right.expression, context),
            _ => Ok(false),
        }
    }
}

impl<T: CssNumericRoot, R: CssRange<T>>
    crate::serialization::specified_rule_serialization::SpecifiedProvider for CssNumeric<T, R>
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}

/// Rejects the first component recovered by implicit closure.
pub(crate) fn reject_recovered(values: &CssComponentValues) -> Result<()> {
    match values.first_implicit_origin() {
        Some(origin) => Err(CssNumericConstructionError::at_origin(
            Kind::RecoveredComponent,
            origin.clone(),
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
pub(crate) mod tests;
