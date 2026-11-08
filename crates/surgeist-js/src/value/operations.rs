use crate::ContextAccess;
use crate::{
    JsResult, JsValue,
    error::JsNativeError,
    value::{
        JsSymbol, PreferredType,
        primitive::{PrimitiveBinaryOp, PrimitiveUnaryOp},
    },
};

impl JsValue {
    /// Perform the binary `+` operator on the value and return the result.
    pub fn add(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Add, other);
        }
        let x = self.to_primitive(context.access(), PreferredType::Default)?;
        let y = other.to_primitive(context.access(), PreferredType::Default)?;
        if x.is_string() || y.is_string() {
            return x.primitive_binary_unmetered(PrimitiveBinaryOp::Add, &y);
        }
        let x = Self::from(x.to_numeric(context.access())?);
        let y = Self::from(y.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Add, &y)
    }

    /// Perform the binary `-` operator on the value and return the result.
    pub fn sub(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Sub, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Sub, &y)
    }

    /// Perform the binary `*` operator on the value and return the result.
    pub fn mul(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Mul, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Mul, &y)
    }

    /// Perform the binary `/` operator on the value and return the result.
    pub fn div(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Div, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Div, &y)
    }

    /// Perform the binary `%` operator on the value and return the result.
    pub fn rem(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Rem, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Rem, &y)
    }

    /// Perform the binary `**` operator on the value and return the result.
    // NOTE: There are some cases in the spec where we have to compare floats
    #[allow(clippy::float_cmp)]
    pub fn pow(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Pow, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Pow, &y)
    }

    /// Perform the binary `&` operator on the value and return the result.
    pub fn bitand(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::BitAnd, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::BitAnd, &y)
    }

    /// Perform the binary `|` operator on the value and return the result.
    pub fn bitor(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::BitOr, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::BitOr, &y)
    }

    /// Perform the binary `^` operator on the value and return the result.
    pub fn bitxor(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::BitXor, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::BitXor, &y)
    }

    /// Perform the binary `<<` operator on the value and return the result.
    pub fn shl(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Shl, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Shl, &y)
    }

    /// Perform the binary `>>` operator on the value and return the result.
    pub fn shr(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Shr, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Shr, &y)
    }

    /// Perform the binary `>>>` operator on the value and return the result.
    pub fn ushr(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_binary_unmetered(PrimitiveBinaryOp::Ushr, other);
        }
        let x = Self::from(self.to_numeric(context.access())?);
        let y = Self::from(other.to_numeric(context.access())?);
        x.primitive_binary_unmetered(PrimitiveBinaryOp::Ushr, &y)
    }

    /// Abstract operation `InstanceofOperator ( V, target )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-instanceofoperator
    pub fn instance_of(&self, target: &Self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        // 1. If Type(target) is not Object, throw a TypeError exception.
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "right-hand side of 'instanceof' should be an object, got `{}`",
                    target.type_of()
                ))
                .into());
        }

        // 2. Let instOfHandler be ? GetMethod(target, @@hasInstance).
        match target.get_method(JsSymbol::has_instance(), context)? {
            // 3. If instOfHandler is not undefined, then
            Some(instance_of_handler) => {
                // a. Return ! ToBoolean(? Call(instOfHandler, target, « V »)).
                Ok(instance_of_handler
                    .call(target, std::slice::from_ref(self), context.access())?
                    .to_boolean())
            }
            None if target.is_callable() => {
                // 5. Return ? OrdinaryHasInstance(target, V).
                Self::ordinary_has_instance(target, self, context.access())
            }
            None => {
                // 4. If IsCallable(target) is false, throw a TypeError exception.
                Err(JsNativeError::typ()
                    .with_message("right-hand side of 'instanceof' is not callable")
                    .into())
            }
        }
    }

    /// Returns the negated value.
    pub fn neg(&self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        if self.is_object() {
            return Ok(Self::new(
                self.to_numeric_number(context.access())
                    .map_or(f64::NAN, std::ops::Neg::neg),
            ));
        }
        self.primitive_unary_unmetered(PrimitiveUnaryOp::Minus)
    }

    /// Returns the negated boolean value.
    #[inline]
    pub fn not(&self) -> JsResult<bool> {
        Ok(!self.to_boolean())
    }

    /// Abstract relational comparison
    ///
    /// The comparison `x < y`, where `x` and `y` are values, produces `true`, `false`,
    /// or `undefined` (which indicates that at least one operand is `NaN`).
    ///
    /// In addition to `x` and `y` the algorithm takes a Boolean flag named `LeftFirst` as a parameter.
    /// The flag is used to control the order in which operations with potentially visible side-effects
    /// are performed upon `x` and `y`. It is necessary because ECMAScript specifies left to right evaluation
    /// of expressions. The default value of `LeftFirst` is `true` and indicates that the `x` parameter
    /// corresponds to an expression that occurs to the left of the `y` parameter's corresponding expression.
    ///
    /// If `LeftFirst` is `false`, the reverse is the case and operations must be performed upon `y` before `x`.
    ///
    /// More Information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-abstract-relational-comparison
    pub fn abstract_relation(
        &self,
        other: &Self,
        left_first: bool,
        context: ContextAccess<'_>,
    ) -> JsResult<AbstractRelation> {
        let context = context.into_owner();
        if !self.is_object() && !other.is_object() {
            return self.primitive_relation_unmetered(other);
        }
        let (x, y) = if left_first {
            let x = self.to_primitive(context.access(), PreferredType::Number)?;
            let y = other.to_primitive(context.access(), PreferredType::Number)?;
            (x, y)
        } else {
            let y = other.to_primitive(context.access(), PreferredType::Number)?;
            let x = self.to_primitive(context.access(), PreferredType::Number)?;
            (x, y)
        };
        x.primitive_relation_unmetered(&y)
    }

    /// The less than operator (`<`) returns `true` if the left operand is less than the right operand,
    /// and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn lt(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        match self.abstract_relation(other, true, context.access())? {
            AbstractRelation::True => Ok(true),
            AbstractRelation::False | AbstractRelation::Undefined => Ok(false),
        }
    }

    /// The less than or equal operator (`<=`) returns `true` if the left operand is less than
    /// or equal to the right operand, and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than_or_equal
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn le(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        match other.abstract_relation(self, false, context.access())? {
            AbstractRelation::False => Ok(true),
            AbstractRelation::True | AbstractRelation::Undefined => Ok(false),
        }
    }

    /// The greater than operator (`>`) returns `true` if the left operand is greater than
    /// the right operand, and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn gt(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        match other.abstract_relation(self, false, context.access())? {
            AbstractRelation::True => Ok(true),
            AbstractRelation::False | AbstractRelation::Undefined => Ok(false),
        }
    }

    /// The greater than or equal operator (`>=`) returns `true` if the left operand is greater than
    /// or equal to the right operand, and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than_or_equal
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn ge(&self, other: &Self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        match self.abstract_relation(other, true, context.access())? {
            AbstractRelation::False => Ok(true),
            AbstractRelation::True | AbstractRelation::Undefined => Ok(false),
        }
    }

    // ==== Numeric Fast Paths ====
    //
    // These methods provide optimized paths for binary operations at the opcode
    // handler level. They use `self.0.as_integer32()` and `self.as_number_cheap()`
    // (pure bit operations) instead of `variant()`, which avoids cloning pointer
    // types (Object, String, Symbol, BigInt) for non-numeric values.
    //
    // Returns `Some(result)` if both operands are numeric, `None` otherwise
    // (caller should fall back to the full method with type coercion).

    /// Converts the value to a number if possible, using fast bit operations. This is
    /// used for the fast operations to allow mixed i32/f64 values.
    #[inline]
    fn as_number_cheap(&self) -> Option<f64> {
        if let Some(i) = self.0.as_integer32() {
            Some(f64::from(i))
        } else {
            self.0.as_float64()
        }
    }

    /// Fast path for the binary `+` operator (numeric only).
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn add_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(
                x.checked_add(y)
                    .map_or_else(|| Self::new(f64::from(x) + f64::from(y)), Self::new),
            );
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new(x + y))
    }

    /// Fast path for the binary `-` operator.
    #[inline]
    pub(crate) fn sub_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(
                x.checked_sub(y)
                    .map_or_else(|| Self::new(f64::from(x) - f64::from(y)), Self::new),
            );
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new(x - y))
    }

    /// Fast path for the binary `*` operator.
    #[inline]
    pub(crate) fn mul_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(
                x.checked_mul(y)
                    .filter(|v| *v != 0 || i32::min(x, y) >= 0)
                    .map_or_else(|| Self::new(f64::from(x) * f64::from(y)), Self::new),
            );
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new(x * y))
    }

    /// Fast path for the binary `/` operator.
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn div_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(
                x.checked_div(y)
                    .filter(|div| y * div == x)
                    .map_or_else(|| Self::new(f64::from(x) / f64::from(y)), Self::new),
            );
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new(x / y))
    }

    /// Fast path for the binary `%` operator.
    #[inline]
    pub(crate) fn rem_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            if y == 0 {
                return Some(Self::nan());
            }
            return Some(match x % y {
                rem if rem == 0 && x < 0 => Self::new(-0.0),
                rem => Self::new(rem),
            });
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new((x % y).copysign(x)))
    }

    /// Fast path for the binary `**` operator.
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn pow_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(
                u32::try_from(y)
                    .ok()
                    .and_then(|y| x.checked_pow(y))
                    .map_or_else(|| Self::new(f64::from(x).powi(y)), Self::new),
            );
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        if x.abs() == 1.0 && y.is_infinite() {
            Some(Self::nan())
        } else {
            Some(Self::new(x.powf(y)))
        }
    }

    /// Fast path for the binary `&` operator (i32 only).
    #[inline]
    pub(crate) fn bitand_fast(&self, other: &Self) -> Option<Self> {
        let x = self.0.as_integer32()?;
        let y = other.0.as_integer32()?;
        Some(Self::new(x & y))
    }

    /// Fast path for the binary `|` operator (i32 only).
    #[inline]
    pub(crate) fn bitor_fast(&self, other: &Self) -> Option<Self> {
        let x = self.0.as_integer32()?;
        let y = other.0.as_integer32()?;
        Some(Self::new(x | y))
    }

    /// Fast path for the binary `^` operator (i32 only).
    #[inline]
    pub(crate) fn bitxor_fast(&self, other: &Self) -> Option<Self> {
        let x = self.0.as_integer32()?;
        let y = other.0.as_integer32()?;
        Some(Self::new(x ^ y))
    }

    /// Fast path for the binary `<<` operator (i32 only).
    #[inline]
    pub(crate) fn shl_fast(&self, other: &Self) -> Option<Self> {
        let x = self.0.as_integer32()?;
        let y = other.0.as_integer32()?;
        Some(Self::new(x.wrapping_shl(y as u32)))
    }

    /// Fast path for the binary `>>` operator (i32 only).
    #[inline]
    pub(crate) fn shr_fast(&self, other: &Self) -> Option<Self> {
        let x = self.0.as_integer32()?;
        let y = other.0.as_integer32()?;
        Some(Self::new(x.wrapping_shr(y as u32)))
    }

    /// Fast path for the binary `>>>` operator (i32 only).
    #[inline]
    pub(crate) fn ushr_fast(&self, other: &Self) -> Option<Self> {
        let x = self.0.as_integer32()?;
        let y = other.0.as_integer32()?;
        Some(Self::new((x as u32).wrapping_shr(y as u32)))
    }

    /// Fast path for the `<` operator.
    #[inline]
    pub(crate) fn lt_fast(&self, other: &Self) -> Option<bool> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(x < y);
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(x < y)
    }

    /// Fast path for the `<=` operator.
    #[inline]
    pub(crate) fn le_fast(&self, other: &Self) -> Option<bool> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(x <= y);
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(x <= y)
    }

    /// Fast path for the `>` operator.
    #[inline]
    pub(crate) fn gt_fast(&self, other: &Self) -> Option<bool> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(x > y);
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(x > y)
    }

    /// Fast path for the `>=` operator.
    #[inline]
    pub(crate) fn ge_fast(&self, other: &Self) -> Option<bool> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(x >= y);
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(x >= y)
    }

    /// Fast path for the `==` operator (numeric only).
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn equals_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(Self::new(x == y));
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new(x == y))
    }

    /// Fast path for the `!=` operator (numeric only).
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn not_equals_fast(&self, other: &Self) -> Option<Self> {
        if let (Some(x), Some(y)) = (self.0.as_integer32(), other.0.as_integer32()) {
            return Some(Self::new(x != y));
        }
        let x = self.as_number_cheap()?;
        let y = other.as_number_cheap()?;
        Some(Self::new(x != y))
    }
}

/// The result of the [Abstract Relational Comparison][arc].
///
/// Comparison `x < y`, where `x` and `y` are values.
/// It produces `true`, `false`, or `undefined`
/// (which indicates that at least one operand is `NaN`).
///
/// [arc]: https://tc39.es/ecma262/#sec-abstract-relational-comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbstractRelation {
    /// `x` is less than `y`
    True,
    /// `x` is **not** less than `y`
    False,
    /// Indicates that at least one operand is `NaN`
    Undefined,
}

impl From<bool> for AbstractRelation {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}
