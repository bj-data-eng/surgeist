//! Bounded specified-value projection; the authored graph is never rewritten.

use super::{
    CssCalculationExpression, CssMathFunction as Function, CssNumericConstant, CssNumericDimension,
    CssNumericType, CssRoundingStrategy, NodeKind,
};
use crate::{
    CssComponentValueRef, CssSpecifiedValueSerializationError as Error,
    CssSpecifiedValueSerializationErrorKind as ErrorKind,
    CssSpecifiedValueSerializationLimits as Limits, CssValueTokenRef,
    specified_serialization::SpecifiedSerializationContext,
};
use std::collections::BTreeMap;

mod math;

type Result<T> = std::result::Result<T, Error>;
type Id = usize;

// MAX = (2^53 - 1) * 2^971. Divisibility by 360 therefore requires the
// significand to be divisible by 45; (2^53 - 1) % 45 == 31. All finite
// binary64 degrees remain supported, including the 31 values above this one.
const ANGLE_OVERFLOW_ENDPOINT: f64 = f64::from_bits(f64::MAX.to_bits() - 31);

fn finite_angle_overflow(value: f64, finite_operands: bool) -> f64 {
    if value.is_infinite() && finite_operands {
        ANGLE_OVERFLOW_ENDPOINT.copysign(value)
    } else {
        value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Unit {
    Number,
    Percentage,
    Canonical(&'static str),
    Context(String),
}
impl Unit {
    fn name(&self) -> &str {
        match self {
            Self::Number => "",
            Self::Percentage => "%",
            Self::Canonical(s) => s,
            Self::Context(s) => s,
        }
    }
    fn resolved(&self) -> bool {
        !matches!(self, Self::Context(_))
    }
    fn nonnegative_basis(&self) -> bool {
        let Self::Context(unit) = self else {
            return true;
        };
        use crate::CssLengthUnit::*;
        // Font size, line height and box sizes can be zero. Glyph metrics and
        // flex fractions require their own basis guarantee before folding.
        matches!(
            crate::CssLengthUnit::from_css_unit(unit),
            Some(
                Em | Rem
                    | Lh
                    | Rlh
                    | Vw
                    | Vh
                    | Vi
                    | Vb
                    | Vmin
                    | Vmax
                    | Svw
                    | Svh
                    | Svi
                    | Svb
                    | Svmin
                    | Svmax
                    | Lvw
                    | Lvh
                    | Lvi
                    | Lvb
                    | Lvmin
                    | Lvmax
                    | Dvw
                    | Dvh
                    | Dvi
                    | Dvb
                    | Dvmin
                    | Dvmax
                    | Cqw
                    | Cqh
                    | Cqi
                    | Cqb
                    | Cqmin
                    | Cqmax
            )
        )
    }
    fn from_type(ty: CssNumericType) -> Self {
        if ty.is_number() {
            return Self::Number;
        }
        if ty.is(CssNumericDimension::Percentage) {
            return Self::Percentage;
        }
        for (dimension, unit) in [
            (CssNumericDimension::Length, "px"),
            (CssNumericDimension::Angle, "deg"),
            (CssNumericDimension::Time, "s"),
            (CssNumericDimension::Frequency, "hz"),
            (CssNumericDimension::Resolution, "dppx"),
        ] {
            if ty.is(dimension) {
                return Self::Canonical(unit);
            }
        }
        // Flex magnitudes require their owning layout context.
        assert!(
            ty.is(CssNumericDimension::Flex),
            "simple projected numeric type"
        );
        Self::Context("fr".into())
    }
}

#[derive(Clone, Debug)]
struct Scalar {
    value: f64,
    unit: Unit,
}
#[derive(Debug)]
enum Kind {
    Scalar(Scalar),
    Sum(Vec<Id>),
    Product(Vec<Id>),
    Negate(Id),
    Invert(Id),
    Function {
        function: Function,
        args: Vec<Option<Id>>,
        strategy: Option<CssRoundingStrategy>,
    },
    Symbol(String),
    // Flattened containers relinquish their owned edge vectors. IDs are never reused.
    Consumed,
}
#[derive(Debug)]
struct Node {
    kind: Kind,
    ty: CssNumericType,
    // Paired with the full checked dimensional exponents in `ty`. A compound
    // dimension need not have a standalone CSS scalar spelling to participate
    // in a surrounding product or inverse. None means external context remains.
    resolved_magnitude: Option<f64>,
}

struct Projection<'a> {
    arena: Vec<Node>,
    context: &'a mut SpecifiedSerializationContext,
}
impl Projection<'_> {
    fn add(&mut self, kind: Kind, ty: CssNumericType) -> Result<Id> {
        self.context.charge_projection(1)?;
        self.arena
            .try_reserve(1)
            .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
        let resolved_magnitude = match &kind {
            Kind::Scalar(value)
                if value.value.is_nan()
                    || value.unit.resolved()
                        && !(matches!(value.unit, Unit::Percentage) && ty.hint.is_some()) =>
            {
                Some(value.value)
            }
            Kind::Product(children) => self.resolved_terms(children, true),
            Kind::Sum(children) => self.resolved_terms(children, false),
            Kind::Negate(child) => self.arena[*child].resolved_magnitude.map(|value| -value),
            Kind::Invert(child) => self.arena[*child]
                .resolved_magnitude
                .map(|value| 1.0 / value),
            _ => None,
        };
        let id = self.arena.len();
        self.arena.push(Node {
            kind,
            ty,
            resolved_magnitude,
        });
        Ok(id)
    }
    // An exceptional numeric subset poisons the operation even when another
    // term still requires external context. No contextual magnitude is guessed.
    fn resolved_terms(&self, children: &[Id], product: bool) -> Option<f64> {
        let mut known = None;
        let mut all_resolved = true;
        for &child in children {
            if let Some(value) = self.arena[child].resolved_magnitude {
                known = Some(match known {
                    Some(previous) if product => previous * value,
                    Some(previous) => previous + value,
                    None => value,
                });
            } else {
                all_resolved = false;
            }
        }
        known.filter(|value| all_resolved || value.is_nan())
    }
    fn is_nan(&self, id: Id) -> bool {
        self.arena[id].resolved_magnitude.is_some_and(f64::is_nan)
    }
    fn scalar(&self, id: Id) -> Option<&Scalar> {
        if let Kind::Scalar(value) = &self.arena[id].kind {
            Some(value)
        } else {
            None
        }
    }
    fn magnitude_comparable(&self, id: Id) -> bool {
        self.scalar(id).is_some_and(|value| {
            // An unknown percentage basis can reverse coefficient ordering.
            !matches!(value.unit, Unit::Percentage) || self.arena[id].ty.hint.is_none()
        })
    }
    fn value(&mut self, value: f64, unit: Unit, ty: CssNumericType) -> Result<Id> {
        self.add(Kind::Scalar(Scalar { value, unit }), ty)
    }
    fn negate(&mut self, id: Id) -> Result<Id> {
        if let Some(scalar) = self.scalar(id).cloned() {
            return self.value(-scalar.value, scalar.unit, self.arena[id].ty);
        }
        if let Kind::Negate(child) = self.arena[id].kind {
            return Ok(child);
        }
        self.add(Kind::Negate(id), self.arena[id].ty)
    }
    fn invert(&mut self, id: Id) -> Result<Id> {
        if let Some(scalar) = self.scalar(id)
            && scalar.unit == Unit::Number
        {
            return self.value(1.0 / scalar.value, Unit::Number, CssNumericType::NUMBER);
        }
        if let Kind::Invert(child) = self.arena[id].kind {
            return Ok(child);
        }
        let ty = CssNumericType::NUMBER
            .product(self.arena[id].ty, true)
            .expect("admitted inverse type");
        self.add(Kind::Invert(id), ty)
    }
    fn sum(&mut self, ids: Vec<Id>, ty: CssNumericType) -> Result<Id> {
        if ty.simple() && ids.iter().any(|&id| self.is_nan(id)) {
            return self.value(f64::NAN, Unit::from_type(ty), ty);
        }
        let mut pending = ids;
        pending.reverse();
        let mut other = Vec::new();
        let mut scalars: BTreeMap<Unit, (f64, bool)> = BTreeMap::new();
        // Visit flattened children in source order, including binary64 sums.
        while let Some(id) = pending.pop() {
            if matches!(self.arena[id].kind, Kind::Sum(_)) {
                let Kind::Sum(children) =
                    std::mem::replace(&mut self.arena[id].kind, Kind::Consumed)
                else {
                    unreachable!()
                };
                pending.extend(children.into_iter().rev());
            } else if let Some(scalar) = self.scalar(id) {
                scalars
                    .entry(scalar.unit.clone())
                    .and_modify(|(value, finite_operands)| {
                        *value += scalar.value;
                        *finite_operands &= scalar.value.is_finite();
                    })
                    .or_insert((scalar.value, scalar.value.is_finite()));
            } else {
                other.push(id);
            }
        }
        if ty.simple() && scalars.values().any(|(value, _)| value.is_nan()) {
            return self.value(f64::NAN, Unit::from_type(ty), ty);
        }
        let mut combined = Vec::new();
        for (unit, (value, finite_operands)) in scalars {
            // Each same-unit set creates one replacement. Range conversion
            // occurs here, after source-order accumulation, not at prefixes.
            // The accumulator may overflow although every source operand was
            // finite; a genuine infinite operand must remain distinguishable.
            let value = if ty.is(CssNumericDimension::Angle) && unit == Unit::Canonical("deg") {
                finite_angle_overflow(value, finite_operands)
            } else {
                value
            };
            combined.push(self.value(value, unit, ty)?);
        }
        combined.extend(other);
        if combined.len() == 1 {
            return Ok(combined[0]);
        }
        self.add(Kind::Sum(combined), ty)
    }
    fn product(&mut self, ids: Vec<Id>, ty: CssNumericType) -> Result<Id> {
        let mut pending = ids;
        let mut flat = Vec::new();
        while let Some(id) = pending.pop() {
            if matches!(self.arena[id].kind, Kind::Product(_)) {
                let Kind::Product(children) =
                    std::mem::replace(&mut self.arena[id].kind, Kind::Consumed)
                else {
                    unreachable!()
                };
                pending.extend(children);
            } else {
                flat.push(id);
            }
        }
        flat.reverse();
        // Product simplification merges Number children before distributing or
        // evaluating the typed product. Accumulate them in source order and
        // retain the first Number's position among the remaining children.
        let mut number: Option<(Id, f64, usize, usize)> = None;
        let mut other = Vec::new();
        for id in flat {
            if let Some(value) = self.scalar(id)
                && value.unit == Unit::Number
            {
                if let Some((_, coefficient, count, _)) = &mut number {
                    *coefficient *= value.value;
                    *count += 1;
                } else {
                    number = Some((id, value.value, 1, other.len()));
                }
            } else {
                other.push(id);
            }
        }
        if let Some((first, coefficient, count, position)) = number {
            let id = if count == 1 {
                first
            } else {
                let number_type = if other.is_empty() {
                    ty
                } else {
                    CssNumericType::NUMBER
                };
                self.value(coefficient, Unit::Number, number_type)?
            };
            if other.len() == 1 {
                let child = other[0];
                if let Kind::Sum(children) = &self.arena[child].kind
                    && children.iter().all(|&id| self.scalar(id).is_some())
                {
                    let children = children.clone();
                    let mut distributed = Vec::new();
                    for id in children {
                        let value = self.scalar(id).expect("checked scalar child").clone();
                        distributed.push(self.value(
                            coefficient * value.value,
                            value.unit,
                            self.arena[id].ty,
                        )?);
                    }
                    return self.sum(distributed, ty);
                }
            }
            other.insert(position, id);
        }
        if other.len() == 1 {
            return Ok(other[0]);
        }
        if ty.simple() {
            let resolved = self.resolved_terms(&other, true);
            if let Some(value) = resolved {
                let value = if ty.is(CssNumericDimension::Angle) {
                    finite_angle_overflow(
                        value,
                        other.iter().all(|&id| {
                            self.arena[id]
                                .resolved_magnitude
                                .is_some_and(f64::is_finite)
                        }),
                    )
                } else {
                    value
                };
                return self.value(value, Unit::from_type(ty), ty);
            }
        }
        if let Some((_, coefficient, _, position)) = number
            && other.len() == 2
        {
            let child = other[1 - position];
            if let Some(value) = self.scalar(child).cloned() {
                return self.value(coefficient * value.value, value.unit, ty);
            }
        }
        self.add(Kind::Product(other), ty)
    }
    fn function(
        &mut self,
        function: Function,
        args: Vec<Option<Id>>,
        strategy: Option<CssRoundingStrategy>,
        ty: CssNumericType,
    ) -> Result<Id> {
        if function == Function::Calc {
            return Ok(args[0].expect("calc argument"));
        }
        // NaN is infectious in every CSS math function, including pow(NaN, 0).
        if args.iter().flatten().any(|&id| self.is_nan(id)) {
            return self.value(f64::NAN, Unit::from_type(ty), ty);
        }
        if function == Function::Clamp && args[0].is_none() && args[2].is_none() {
            return Ok(args[1].expect("clamp value argument"));
        }
        let values: Option<Vec<Option<Scalar>>> = args
            .iter()
            .map(|id| match id {
                Some(id) => self.scalar(*id).cloned().map(Some),
                None => Some(None),
            })
            .collect();
        if let Some(values) = values {
            let resolved = args
                .iter()
                .flatten()
                .all(|&id| self.arena[id].resolved_magnitude.is_some());
            let same_unit = values
                .iter()
                .flatten()
                .map(|v| &v.unit)
                .all(|unit| Some(unit) == values.iter().flatten().next().map(|v| &v.unit));
            if resolved
                || same_unit
                    && (matches!(function, Function::Min | Function::Max | Function::Clamp)
                        || matches!(function, Function::Abs | Function::Hypot)
                            && values
                                .iter()
                                .flatten()
                                .all(|value| value.unit.nonnegative_basis()))
                    && args
                        .iter()
                        .flatten()
                        .all(|&id| self.magnitude_comparable(id))
            {
                let output = math::evaluate(function, &values, strategy);
                let output = if ty.is(CssNumericDimension::Angle) {
                    finite_angle_overflow(
                        output,
                        values.iter().flatten().all(|value| value.value.is_finite()),
                    )
                } else {
                    output
                };
                let unit = if resolved {
                    Unit::from_type(ty)
                } else {
                    values
                        .iter()
                        .flatten()
                        .next()
                        .expect("function argument")
                        .unit
                        .clone()
                };
                return self.value(output, unit, ty);
            }
        }
        if matches!(function, Function::Min | Function::Max) {
            let mut grouped: BTreeMap<Unit, (usize, f64)> = BTreeMap::new();
            let mut other = Vec::new();
            for id in args.into_iter().flatten() {
                if let Some(scalar) = self.scalar(id)
                    && self.magnitude_comparable(id)
                {
                    if let Some((_, value)) = grouped.get_mut(&scalar.unit) {
                        *value = math::extreme(*value, scalar.value, function == Function::Min);
                    } else {
                        grouped.insert(scalar.unit.clone(), (other.len(), scalar.value));
                        other.push(Some(id));
                    }
                } else {
                    other.push(Some(id));
                }
            }
            for (unit, (position, value)) in grouped {
                other[position] = Some(self.value(value, unit, ty)?);
            }
            if other.len() == 1 {
                return Ok(other[0].expect("present comparison argument"));
            }
            return self.add(
                Kind::Function {
                    function,
                    args: other,
                    strategy,
                },
                ty,
            );
        }
        self.add(
            Kind::Function {
                function,
                args,
                strategy,
            },
            ty,
        )
    }
}

/// Projects a checked expression without modifying its syntax or provenance.
pub(crate) fn project_specified(
    expression: &CssCalculationExpression,
    limits: Limits,
) -> Result<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let mut output = String::new();
    project_specified_into(expression, &mut context, &mut output)?;
    Ok(output)
}

/// A private summary used by owning color slots to distinguish a fully numeric
/// result from retained context without reparsing the emitted CSS.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NumericProjectionOutcome {
    pub(crate) context_dependent: bool,
    pub(crate) scalar_value: Option<f64>,
}

/// Slot-owned dimensional conversion applied to a projected calculation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericProjectionScale {
    Identity,
    Number { numerator: u64, denominator: u64 },
    PercentageToNumber { numerator: u64, denominator: u64 },
    NumberToPercentage { numerator: u64, denominator: u64 },
}

/// Projects into one caller-owned output and cumulative resource context.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn project_specified_into(
    expression: &CssCalculationExpression,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> Result<NumericProjectionOutcome> {
    project_specified_impl(
        expression,
        NumericProjectionScale::Identity,
        context,
        output,
        true,
    )
}

/// Projects a calc-sum in a grammar that supplies its own outer function.
/// This suppresses only the numeric serializer's calc wrapper.
pub(crate) fn project_calc_size_sum_into(
    expression: &CssCalculationExpression,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> Result<NumericProjectionOutcome> {
    project_specified_impl_mode(
        expression,
        NumericProjectionScale::Identity,
        context,
        output,
        (true, false),
        NumericEmission::CssComponent(None),
    )
}

/// Projects one child into bounded scratch storage for caller-side branch
/// selection. Input and projection work remain cumulative; final-output bytes
/// are charged only if the caller later appends the returned text.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn capture_specified(
    expression: &CssCalculationExpression,
    context: &mut SpecifiedSerializationContext,
) -> Result<(String, NumericProjectionOutcome)> {
    capture_specified_scaled(expression, NumericProjectionScale::Identity, context)
}

/// Captures generic specified coefficients with the owning slot's dimensional
/// scale, keeping arithmetic and cumulative work in the shared projector.
pub(crate) fn capture_specified_scaled(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
) -> Result<(String, NumericProjectionOutcome)> {
    let mut output = String::new();
    let outcome = project_specified_impl_mode(
        expression,
        scale,
        context,
        &mut output,
        (false, true),
        NumericEmission::CssComponent(None),
    )?;
    Ok((output, outcome))
}

/// Private numeric-emission control preserves finite text and scratch behavior.
#[cfg(test)]
pub(super) fn capture_color_specified_scaled(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
) -> Result<(String, NumericProjectionOutcome)> {
    let mut output = String::new();
    let outcome = project_specified_impl_mode(
        expression,
        scale,
        context,
        &mut output,
        (false, true),
        NumericEmission::ColorCalculation,
    )?;
    Ok((output, outcome))
}

/// Captures ordinary non-alpha color components, selecting coefficient text
/// from the existing post-scale mathematical outcome before emitting scratch.
pub(crate) fn capture_color_component_specified_scaled(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
) -> Result<(String, NumericProjectionOutcome)> {
    let mut output = String::new();
    let outcome = project_specified_impl_mode(
        expression,
        scale,
        context,
        &mut output,
        (false, true),
        NumericEmission::ColorComponentCalculation,
    )?;
    Ok((output, outcome))
}

/// Captures canonical components before lossy finite coefficient formatting.
pub(crate) fn capture_specified_for_comparison(
    expression: &CssCalculationExpression,
    context: &mut SpecifiedSerializationContext,
) -> Result<(NumericComparisonCapture, NumericProjectionOutcome)> {
    let mut output = String::new();
    let mut collector = FiniteCoefficientCollector {
        spans: Vec::new(),
        bound: context.remaining_bytes(),
    };
    let outcome = project_specified_impl_mode(
        expression,
        NumericProjectionScale::Identity,
        context,
        &mut output,
        (false, true),
        NumericEmission::CssComponent(Some(&mut collector)),
    )?;
    Ok((
        NumericComparisonCapture {
            css: output,
            finite_coefficients: collector.spans,
        },
        outcome,
    ))
}

fn project_specified_impl(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    charge_output: bool,
) -> Result<NumericProjectionOutcome> {
    project_specified_impl_mode(
        expression,
        scale,
        context,
        output,
        (charge_output, true),
        NumericEmission::CssComponent(None),
    )
}

fn project_specified_impl_mode(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    (charge_output, outer_calc): (bool, bool),
    mut emission: NumericEmission<'_>,
) -> Result<NumericProjectionOutcome> {
    let mut projection = Projection {
        arena: Vec::new(),
        context,
    };
    let mut stack = vec![(expression, false)];
    let mut results = Vec::new();
    projection.context.charge_input(1)?;
    while let Some((node, finish)) = stack.pop() {
        if !finish {
            let child_count = match &node.kind {
                NodeKind::Group(_) => 1,
                NodeKind::Sum(children) => children.len(),
                NodeKind::Product(children) => children.len(),
                NodeKind::Function { args, .. } => args.iter().flatten().count(),
                _ => 0,
            };
            projection.context.charge_input(child_count)?;
            let children: Vec<_> = match &node.kind {
                NodeKind::Group(child) => vec![child.as_ref()],
                NodeKind::Sum(children) => children.iter().map(|(_, child)| child).collect(),
                NodeKind::Product(children) => children.iter().map(|(_, child)| child).collect(),
                NodeKind::Function { args, .. } => args.iter().flatten().collect(),
                _ => Vec::new(),
            };
            stack.push((node, true));
            stack.extend(children.into_iter().rev().map(|child| (child, false)));
            continue;
        }
        let id = match &node.kind {
            NodeKind::Value(component) => {
                let (number, unit) = match component.view() {
                    CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
                        (number, Unit::Number)
                    }
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => {
                        (number, Unit::Percentage)
                    }
                    CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                        let (unit, factor) = canonical_unit(unit);
                        let value = if unit == Unit::Canonical("deg") {
                            finite_angle_leaf(number.representation(), factor)
                        } else {
                            lexical_value(number.representation()) * factor
                        };
                        let id = projection.value(value, unit, node.ty)?;
                        results.push(id);
                        continue;
                    }
                    _ => unreachable!("checked numeric leaf"),
                };
                projection.value(lexical_value(number.representation()), unit, node.ty)?
            }
            NodeKind::Constant(constant) => projection.value(
                match constant {
                    CssNumericConstant::E => std::f64::consts::E,
                    CssNumericConstant::Pi => std::f64::consts::PI,
                    CssNumericConstant::Infinity => f64::INFINITY,
                    CssNumericConstant::NegativeInfinity => f64::NEG_INFINITY,
                    CssNumericConstant::NaN => f64::NAN,
                },
                Unit::Number,
                node.ty,
            )?,
            NodeKind::Size => projection.add(Kind::Symbol("size".into()), node.ty)?,
            NodeKind::ProfileChannel(name) => {
                let name = super::capture_identifier(name.as_str(), projection.context)?;
                projection.add(Kind::Symbol(name), node.ty)?
            }
            NodeKind::Variable(channel) => projection.add(
                Kind::Symbol(format!("{channel:?}").to_ascii_lowercase()),
                node.ty,
            )?,
            NodeKind::TreeCounting(function) => {
                projection.add(Kind::Symbol(format!("{}()", function.name())), node.ty)?
            }
            NodeKind::Group(_) => results.pop().expect("postorder group operand"),
            NodeKind::Sum(children) => {
                let start = results.len() - children.len();
                let mut ids = results.split_off(start);
                for ((operator, _), id) in children.iter().zip(&mut ids) {
                    if matches!(operator, Some(super::CssCalculationSumOperator::Subtract)) {
                        *id = projection.negate(*id)?;
                    }
                }
                projection.sum(ids, node.ty)?
            }
            NodeKind::Product(children) => {
                let start = results.len() - children.len();
                let mut ids = results.split_off(start);
                for ((operator, _), id) in children.iter().zip(&mut ids) {
                    if matches!(operator, Some(super::CssCalculationProductOperator::Divide)) {
                        *id = projection.invert(*id)?;
                    }
                }
                projection.product(ids, node.ty)?
            }
            NodeKind::Function {
                function,
                args,
                strategy,
            } => {
                let start = results.len() - args.iter().flatten().count();
                let mut ids = results.split_off(start).into_iter();
                let args = args
                    .iter()
                    .map(|arg| {
                        arg.as_ref()
                            .map(|_| ids.next().expect("postorder function argument"))
                    })
                    .collect();
                projection.function(*function, args, *strategy, node.ty)?
            }
        };
        results.push(id);
    }
    let root = apply_scale(&mut projection, results[0], scale)?;
    let outcome = NumericProjectionOutcome {
        context_dependent: projection.arena[root].resolved_magnitude.is_none(),
        scalar_value: projection.scalar(root).map(|value| value.value),
    };
    emission = match emission {
        NumericEmission::ColorComponentCalculation if outcome.context_dependent => {
            NumericEmission::CssComponent(None)
        }
        NumericEmission::ColorComponentCalculation => NumericEmission::ColorCalculation,
        emission => emission,
    };
    // Discarded values still perform the same checked projection and arithmetic.
    // Their formatting does not consume the surviving value's output budget.
    if projection.context.output_suppressed() {
        return Ok(outcome);
    }
    if outer_calc {
        projection.serialize(root, output, charge_output, &mut emission)?;
    } else {
        projection.serialize_mode(root, output, charge_output, false, &mut emission)?;
    }
    Ok(outcome)
}

fn apply_scale(
    projection: &mut Projection<'_>,
    root: Id,
    scale: NumericProjectionScale,
) -> Result<Id> {
    let number = CssNumericType::NUMBER;
    let percentage = CssNumericType::dimension(CssNumericDimension::Percentage);
    let (numerator, denominator, unit, target) = match scale {
        NumericProjectionScale::Identity => return Ok(root),
        NumericProjectionScale::Number {
            numerator,
            denominator,
        } => (numerator, denominator, Unit::Number, number),
        NumericProjectionScale::PercentageToNumber {
            numerator,
            denominator,
        } => (denominator, numerator, Unit::Percentage, number),
        NumericProjectionScale::NumberToPercentage {
            numerator,
            denominator,
        } => (numerator, denominator, Unit::Percentage, percentage),
    };
    debug_assert_ne!(denominator, 0);
    let factor = numerator as f64 / denominator as f64;
    let factor_type = if matches!(unit, Unit::Percentage) {
        percentage
    } else {
        number
    };
    let factor = projection.value(factor, unit, factor_type)?;
    let factor = if matches!(scale, NumericProjectionScale::PercentageToNumber { .. }) {
        // Cancel the percentage dimension even when the root remains symbolic.
        projection.invert(factor)?
    } else {
        factor
    };
    projection.product(vec![root, factor], target)
}

fn lexical_value(representation: &str) -> f64 {
    if representation
        .split(['e', 'E'])
        .next()
        .expect("numeric mantissa")
        .bytes()
        .all(|b| matches!(b, b'0' | b'+' | b'-' | b'.'))
    {
        0.0
    } else {
        representation
            .parse::<f64>()
            .expect("checked CSS numeric spelling")
    }
}

fn finite_angle_leaf(representation: &str, factor: f64) -> f64 {
    let coefficient = lexical_value(representation);
    let converted = if coefficient.is_infinite() && factor < 1.0 {
        // A grad coefficient outside binary64 can still fit in degrees. Read
        // a bounded normalized decimal prefix before applying its 9/10 scale;
        // neither the authored token nor its exponent is materialized anew.
        let lexical = crate::exact_decimal::LexicalDecimal::new(representation);
        let point = lexical
            .exponent
            .map(|exponent| exponent.saturating_add(lexical.len as i128));
        if point == Some(309) {
            let mut prefix = 0u64;
            let mut digits = 0;
            for digit in lexical.digits().take(19) {
                prefix = prefix * 10 + u64::from(digit);
                digits += 1;
            }
            let normalized = prefix as f64 / 10f64.powi(digits - 1);
            (normalized * factor * 1e308).copysign(coefficient)
        } else {
            coefficient
        }
    } else {
        coefficient * factor
    };
    // Dimension tokens are authored finite decimals. Genuine infinities are
    // constants or calculation results and never enter this leaf conversion.
    finite_angle_overflow(converted, true)
}

fn canonical_unit(unit: &str) -> (Unit, f64) {
    let lower = unit.to_ascii_lowercase();
    let (canonical, factor) = match lower.as_str() {
        "px" => ("px", 1.0),
        "in" => ("px", 96.0),
        "cm" => ("px", 96.0 / 2.54),
        "mm" => ("px", 96.0 / 25.4),
        "q" => ("px", 96.0 / 101.6),
        "pt" => ("px", 96.0 / 72.0),
        "pc" => ("px", 16.0),
        "deg" => ("deg", 1.0),
        "grad" => ("deg", 0.9),
        "rad" => ("deg", 180.0 / std::f64::consts::PI),
        "turn" => ("deg", 360.0),
        "s" => ("s", 1.0),
        "ms" => ("s", 0.001),
        "hz" => ("hz", 1.0),
        "khz" => ("hz", 1000.0),
        "dppx" | "x" => ("dppx", 1.0),
        "dpi" => ("dppx", 1.0 / 96.0),
        "dpcm" => ("dppx", 2.54 / 96.0),
        _ => return (Unit::Context(lower), 1.0),
    };
    (Unit::Canonical(canonical), factor)
}

#[derive(Clone, Copy)]
enum Position {
    Root,
    Argument,
    Operand,
}
enum Output {
    Node(Id, Position),
    Text(String),
    Scalar(Id, bool, bool),
    FinitePrefix(&'static str, f64, usize),
}

impl Projection<'_> {
    fn serialize(
        &mut self,
        root: Id,
        output: &mut String,
        charge_output: bool,
        emission: &mut NumericEmission<'_>,
    ) -> Result<()> {
        self.serialize_mode(root, output, charge_output, true, emission)
    }

    fn serialize_mode(
        &mut self,
        root: Id,
        output: &mut String,
        charge_output: bool,
        outer_calc: bool,
        emission: &mut NumericEmission<'_>,
    ) -> Result<()> {
        let mut work = Vec::new();
        if !outer_calc
            || matches!(
                self.arena[root].kind,
                Kind::Function { .. } | Kind::Symbol(_)
            )
        {
            work.push(Output::Node(root, Position::Root));
        } else {
            work.push(Output::Text(")".into()));
            work.push(Output::Node(root, Position::Root));
            work.push(Output::Text("calc(".into()));
        }
        while let Some(item) = work.pop() {
            let (id, position) = match item {
                Output::Text(text) => {
                    append_piece(self.context, output, &text, charge_output, emission, &[])?;
                    continue;
                }
                Output::Scalar(id, root, negate) => {
                    if charge_output && self.context.output_suppressed() {
                        continue;
                    }
                    let limit = if charge_output {
                        self.context.remaining_bytes()
                    } else {
                        self.context.remaining_bytes().saturating_sub(output.len())
                    };
                    let piece = emission.scalar_piece(
                        self.scalar(id).expect("scalar emission"),
                        root,
                        negate,
                        limit,
                    )?;
                    append_piece(
                        self.context,
                        output,
                        &piece.css,
                        charge_output,
                        emission,
                        &piece.spans[..piece.count],
                    )?;
                    continue;
                }
                Output::FinitePrefix(text, value, end) => {
                    let spans = [FiniteCoefficientSpan {
                        start: 0,
                        end,
                        bits: value.to_bits(),
                    }];
                    append_piece(self.context, output, text, charge_output, emission, &spans)?;
                    continue;
                }
                Output::Node(id, position) => (id, position),
            };
            let mut next = Vec::new();
            match &self.arena[id].kind {
                Kind::Scalar(_) => next.push(Output::Scalar(
                    id,
                    matches!(position, Position::Root),
                    false,
                )),
                Kind::Symbol(text) => next.push(Output::Text(text.clone())),
                Kind::Function {
                    function,
                    args,
                    strategy,
                } => {
                    next.push(Output::Text(format!("{}(", function.name())));
                    if *function == Function::Round
                        && let Some(strategy) = strategy
                        && *strategy != CssRoundingStrategy::Nearest
                    {
                        next.push(Output::Text(format!("{}, ", strategy.name())));
                    }
                    for (index, arg) in args.iter().enumerate() {
                        if index != 0 {
                            next.push(Output::Text(", ".into()));
                        }
                        next.push(match arg {
                            Some(child) => Output::Node(*child, Position::Argument),
                            None => Output::Text("none".into()),
                        });
                    }
                    next.push(Output::Text(")".into()));
                }
                Kind::Sum(children) | Kind::Product(children) => {
                    let sum = matches!(self.arena[id].kind, Kind::Sum(_));
                    let parentheses = matches!(position, Position::Operand);
                    if parentheses {
                        next.push(Output::Text("(".into()));
                    }
                    let mut sorted = children.clone();
                    sorted.sort_by(|&a, &b| self.sort_key(a).cmp(&self.sort_key(b)));
                    for (index, child) in sorted.into_iter().enumerate() {
                        let negative =
                            sum && index != 0 && matches!(self.arena[child].kind, Kind::Negate(_));
                        let negative_scalar = sum
                            && index != 0
                            && self.scalar(child).is_some_and(|value| value.value < 0.0);
                        let inverse = !sum && matches!(self.arena[child].kind, Kind::Invert(_));
                        if index != 0 {
                            next.push(Output::Text(
                                if sum {
                                    if negative || negative_scalar {
                                        " - "
                                    } else {
                                        " + "
                                    }
                                } else if inverse {
                                    " / "
                                } else {
                                    " * "
                                }
                                .into(),
                            ));
                        }
                        if negative {
                            let Kind::Negate(operand) = self.arena[child].kind else {
                                unreachable!()
                            };
                            next.push(Output::Node(operand, Position::Operand));
                        } else if negative_scalar {
                            next.push(Output::Scalar(child, false, true));
                        } else if inverse && index != 0 {
                            let Kind::Invert(operand) = self.arena[child].kind else {
                                unreachable!()
                            };
                            next.push(Output::Node(operand, Position::Operand));
                        } else {
                            next.push(Output::Node(child, Position::Operand));
                        }
                    }
                    if parentheses {
                        next.push(Output::Text(")".into()));
                    }
                }
                Kind::Negate(child) | Kind::Invert(child) => {
                    let parentheses = matches!(position, Position::Operand);
                    if parentheses {
                        next.push(Output::Text("(".into()));
                    }
                    next.push(if matches!(self.arena[id].kind, Kind::Negate(_)) {
                        Output::FinitePrefix("-1 * ", -1.0, 2)
                    } else {
                        Output::FinitePrefix("1 / ", 1.0, 1)
                    });
                    next.push(Output::Node(*child, Position::Operand));
                    if parentheses {
                        next.push(Output::Text(")".into()));
                    }
                }
                Kind::Consumed => unreachable!("flattened container is no longer reachable"),
            }
            work.extend(next.into_iter().rev());
        }
        Ok(())
    }
    fn sort_key(&self, id: Id) -> (u8, &str) {
        match self.scalar(id).map(|value| &value.unit) {
            Some(Unit::Number) => (0, ""),
            Some(Unit::Percentage) => (1, ""),
            Some(unit) => (2, unit.name()),
            None => (3, ""),
        }
    }
}

#[derive(Clone, Copy)]
struct FiniteCoefficientSpan {
    start: usize,
    end: usize,
    bits: u64,
}

pub(crate) struct NumericComparisonCapture {
    css: String,
    finite_coefficients: Vec<FiniteCoefficientSpan>,
}
impl NumericComparisonCapture {
    pub(crate) fn as_css(&self) -> &str {
        &self.css
    }
    pub(crate) fn same_projected_components(&self, other: &Self) -> bool {
        if self.finite_coefficients.len() != other.finite_coefficients.len() {
            return false;
        }
        let (mut left, mut right) = (0, 0);
        for (a, b) in self
            .finite_coefficients
            .iter()
            .zip(&other.finite_coefficients)
        {
            if self.css[left..a.start] != other.css[right..b.start] || a.bits != b.bits {
                return false;
            }
            left = a.end;
            right = b.end;
        }
        self.css[left..] == other.css[right..]
    }
}

// Every span occupies at least one distinct byte of already bounded scratch
// CSS. Thus count <= scratch length <= bound, including repeated tree IDs.
// Only selected comparison captures construct this collector.
struct FiniteCoefficientCollector {
    spans: Vec<FiniteCoefficientSpan>,
    bound: usize,
}
impl FiniteCoefficientCollector {
    fn append(
        &mut self,
        offset: usize,
        length: usize,
        local: &[FiniteCoefficientSpan],
    ) -> Result<()> {
        for span in local {
            let rebased = rebase_span(*span, offset, length)?;
            debug_assert!(
                self.spans
                    .last()
                    .is_none_or(|prior| prior.end <= rebased.start)
            );
            let needed = self
                .spans
                .len()
                .checked_add(1)
                .ok_or_else(|| Error::new(ErrorKind::CapacityOverflow))?;
            if needed > self.spans.capacity() {
                let target = span_capacity(self.spans.len(), needed, self.bound)?;
                self.spans
                    .try_reserve_exact(target - self.spans.len())
                    .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
            }
            self.spans.push(rebased);
        }
        Ok(())
    }
}
fn span_capacity(len: usize, needed: usize, bound: usize) -> Result<usize> {
    if needed > bound {
        return Err(Error::new(ErrorKind::CapacityOverflow));
    }
    let target = len.checked_mul(2).unwrap_or(bound).min(bound).max(needed);
    let bytes = target
        .checked_mul(std::mem::size_of::<FiniteCoefficientSpan>())
        .ok_or_else(|| Error::new(ErrorKind::CapacityOverflow))?;
    if bytes > isize::MAX as usize {
        return Err(Error::new(ErrorKind::CapacityOverflow));
    }
    Ok(target)
}
fn rebase_span(
    span: FiniteCoefficientSpan,
    offset: usize,
    length: usize,
) -> Result<FiniteCoefficientSpan> {
    if span.start >= span.end || span.end > length {
        return Err(Error::new(ErrorKind::CapacityOverflow));
    }
    Ok(FiniteCoefficientSpan {
        start: offset
            .checked_add(span.start)
            .ok_or_else(|| Error::new(ErrorKind::CapacityOverflow))?,
        end: offset
            .checked_add(span.end)
            .ok_or_else(|| Error::new(ErrorKind::CapacityOverflow))?,
        bits: span.bits,
    })
}

enum NumericEmission<'a> {
    CssComponent(Option<&'a mut FiniteCoefficientCollector>),
    ColorCalculation,
    ColorComponentCalculation,
}
// Known scalar syntax contributes at most two coefficients (operand -0).
// These local slots do not allocate and never infer coefficients from text.
struct ScalarPiece {
    css: String,
    spans: [FiniteCoefficientSpan; 2],
    count: usize,
}
impl NumericEmission<'_> {
    fn scalar_piece(
        &self,
        scalar: &Scalar,
        root: bool,
        negate: bool,
        limit: usize,
    ) -> Result<ScalarPiece> {
        let value = if negate { -scalar.value } else { scalar.value };
        let mut spans = [FiniteCoefficientSpan {
            start: 0,
            end: 0,
            bits: 0,
        }; 2];
        if matches!(self, Self::ColorCalculation) {
            return Ok(ScalarPiece {
                css: scalar_text(scalar, root, value),
                spans,
                count: 0,
            });
        }
        let annotate = matches!(self, Self::CssComponent(Some(_)));
        let unit = scalar.unit.name();
        let (css, count) = if value.is_finite() {
            if value == 0.0 && value.is_sign_negative() && !root {
                let css = bounded_piece(&["(0", unit, " * -1)"], limit)?;
                if annotate {
                    spans[0] = FiniteCoefficientSpan {
                        start: 1,
                        end: 2,
                        bits: 0.0_f64.to_bits(),
                    };
                    spans[1] = FiniteCoefficientSpan {
                        start: 5 + unit.len(),
                        end: 7 + unit.len(),
                        bits: (-1.0_f64).to_bits(),
                    };
                }
                (css, usize::from(annotate) * 2)
            } else {
                let value = if value == 0.0 { 0.0 } else { value };
                let coefficient_limit = limit
                    .checked_sub(unit.len())
                    .ok_or_else(|| Error::new(ErrorKind::ByteLimit))?;
                let mut css =
                    crate::numeric_formatting::format_projected_number(value, coefficient_limit)?;
                let end = css.len();
                css.try_reserve(unit.len())
                    .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
                css.push_str(unit);
                spans[0] = FiniteCoefficientSpan {
                    start: 0,
                    end,
                    bits: value.to_bits(),
                };
                (css, usize::from(annotate))
            }
        } else {
            let keyword = if value.is_nan() {
                "NaN"
            } else if value.is_sign_negative() {
                "-infinity"
            } else {
                "infinity"
            };
            if unit.is_empty() {
                (bounded_piece(&[keyword], limit)?, 0)
            } else {
                let prefix = if root { "" } else { "(" };
                let suffix = if root { "" } else { ")" };
                let css = bounded_piece(&[prefix, keyword, " * 1", unit, suffix], limit)?;
                let start = prefix.len() + keyword.len() + 3;
                spans[0] = FiniteCoefficientSpan {
                    start,
                    end: start + 1,
                    bits: 1.0_f64.to_bits(),
                };
                (css, usize::from(annotate))
            }
        };
        Ok(ScalarPiece { css, spans, count })
    }
}
fn bounded_piece(parts: &[&str], limit: usize) -> Result<String> {
    let length = parts
        .iter()
        .try_fold(0_usize, |total, part| total.checked_add(part.len()))
        .ok_or_else(|| Error::new(ErrorKind::CapacityOverflow))?;
    if length > limit {
        return Err(Error::new(ErrorKind::ByteLimit));
    }
    let mut css = String::new();
    css.try_reserve(length)
        .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
    for part in parts {
        css.push_str(part);
    }
    Ok(css)
}
fn append_piece(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    text: &str,
    charge_output: bool,
    emission: &mut NumericEmission<'_>,
    spans: &[FiniteCoefficientSpan],
) -> Result<()> {
    let offset = output.len();
    if charge_output {
        context.append(output, text)?;
    } else {
        context.append_temporary(output, text)?;
    }
    if let NumericEmission::CssComponent(Some(collector)) = emission {
        collector.append(offset, text.len(), spans)?;
    }
    Ok(())
}

fn scalar_text(scalar: &Scalar, root: bool, value: f64) -> String {
    let unit = scalar.unit.name();
    if value.is_finite() {
        if value == 0.0 && value.is_sign_negative() && !root {
            return format!("(0{unit} * -1)");
        }
        return format!(
            "{}{unit}",
            crate::specified_serialization::format_binary64(value)
        );
    }
    let keyword = if value.is_nan() {
        "NaN"
    } else if value.is_sign_negative() {
        "-infinity"
    } else {
        "infinity"
    };
    if unit.is_empty() {
        return keyword.into();
    }
    if root {
        format!("{keyword} * 1{unit}")
    } else {
        format!("({keyword} * 1{unit})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssNumberCalculation, CssPercentageCalculation, parse_component_values};

    fn projected(source: &str) -> String {
        let components = parse_component_values(source).unwrap();
        if let Ok(value) = CssNumberCalculation::try_from_components(components.clone()) {
            project_specified(&value.expression, Limits::default()).unwrap()
        } else {
            let value = CssPercentageCalculation::try_from_components(components).unwrap();
            project_specified(&value.expression, Limits::default()).unwrap()
        }
    }

    #[test]
    fn specified_math_reduces_every_admitted_function_family() {
        for (source, expected) in [
            ("calc(1 / 2)", "calc(0.5)"),
            ("calc(25% + 25%)", "calc(50%)"),
            ("min(25%, 50%)", "calc(25%)"),
            ("max(-1, 2)", "calc(2)"),
            ("clamp(2, 1, 0)", "calc(2)"),
            ("clamp(none, 2, none)", "calc(2)"),
            ("round(-2.5)", "calc(-2)"),
            ("mod(-18, 5)", "calc(2)"),
            ("rem(-18, 5)", "calc(-3)"),
            ("sin(90deg)", "calc(1)"),
            ("cos(0)", "calc(1)"),
            ("tan(90deg)", "calc(infinity)"),
            ("calc(asin(1) / 90deg)", "calc(1)"),
            ("calc(acos(1) / 1deg)", "calc(0)"),
            ("calc(atan(infinity) / 90deg)", "calc(1)"),
            ("calc(atan2(1, 0) / 90deg)", "calc(1)"),
            ("pow(2, 3)", "calc(8)"),
            ("sqrt(4)", "calc(2)"),
            ("hypot(3, 4)", "calc(5)"),
            ("log(1, 2)", "calc(0)"),
            ("exp(0)", "calc(1)"),
            ("abs(-2)", "calc(2)"),
            ("sign(-25%)", "calc(-1)"),
            ("calc(1s / 1000ms)", "calc(1)"),
            ("calc(1in / 96px)", "calc(1)"),
            ("calc(50% / 25%)", "calc(2)"),
        ] {
            assert_eq!(projected(source), expected, "{source}");
        }
    }

    #[test]
    fn exceptional_values_are_closed_typed_math_without_computed_clamping() {
        for (source, expected) in [
            ("calc(1 / 0)", "calc(infinity)"),
            ("calc(-1 / 0)", "calc(-infinity)"),
            ("calc(0 / 0)", "calc(NaN)"),
            ("calc(infinity - infinity)", "calc(NaN)"),
            ("calc(0 * infinity)", "calc(NaN)"),
            ("pow(NaN, 0)", "calc(NaN)"),
            ("hypot(infinity, NaN)", "calc(NaN)"),
            ("calc(infinity * 1%)", "calc(infinity * 1%)"),
            ("calc(NaN * 1%)", "calc(NaN * 1%)"),
            ("calc(1 / (0 * -1))", "calc(-infinity)"),
            ("calc(0 * -1)", "calc(0)"),
            ("calc(150%)", "calc(150%)"),
            ("calc(2 - 3)", "calc(-1)"),
            ("calc(1 / min(0, 0 * -1))", "calc(-infinity)"),
            ("calc(1 / max(0, 0 * -1))", "calc(infinity)"),
        ] {
            assert_eq!(projected(source), expected, "{source}");
        }
    }

    #[test]
    fn context_dependent_values_remain_symbolic_and_canonical() {
        for (source, expected) in [
            ("sign(1em - 1px)", "sign(1em - 1px)"),
            ("calc(1em / 1px)", "calc(1em / 1px)"),
            ("sign(2 * (1em + 1px))", "sign(2em + 2px)"),
        ] {
            assert_eq!(projected(source), expected, "{source}");
        }
    }

    #[test]
    fn limits_fail_atomically_and_do_not_change_the_authored_tree() {
        let value = CssNumberCalculation::try_from_components(
            parse_component_values("calc(1 / 2)").unwrap(),
        )
        .unwrap();
        let before = value.clone();
        for (limits, kind) in [
            (Limits::new(0, 100, 100), ErrorKind::InputNodeLimit),
            (Limits::new(100, 0, 100), ErrorKind::ProjectionNodeLimit),
            (Limits::new(100, 100, 8), ErrorKind::ByteLimit),
        ] {
            assert_eq!(
                project_specified(&value.expression, limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(
            project_specified(&value.expression, Limits::new(100, 100, 9)).unwrap(),
            "calc(0.5)"
        );
        assert_eq!(value, before);
    }

    #[test]
    fn shared_context_charges_sequential_numeric_children_cumulatively() {
        let value = CssNumberCalculation::try_from_components(
            parse_component_values("calc(1 / 2)").unwrap(),
        )
        .unwrap();

        // The checked tree is the calc group, product, and two leaves.
        let mut input_context = SpecifiedSerializationContext::new(Limits::new(4, 100, 100));
        let mut output = String::new();
        project_specified_into(&value.expression, &mut input_context, &mut output).unwrap();
        assert_eq!(output, "calc(0.5)");
        assert_eq!(
            project_specified_into(&value.expression, &mut input_context, &mut output)
                .unwrap_err()
                .kind(),
            ErrorKind::InputNodeLimit
        );

        let mut projection_context = SpecifiedSerializationContext::new(Limits::new(100, 4, 100));
        let mut output = String::new();
        project_specified_into(&value.expression, &mut projection_context, &mut output).unwrap();
        assert_eq!(
            project_specified_into(&value.expression, &mut projection_context, &mut output,)
                .unwrap_err()
                .kind(),
            ErrorKind::ProjectionNodeLimit
        );
    }

    #[test]
    fn shared_context_emits_into_one_bounded_output_and_reports_dependence() {
        let value = CssNumberCalculation::try_from_components(
            parse_component_values("calc(1 / 2)").unwrap(),
        )
        .unwrap();
        let mut context = SpecifiedSerializationContext::new(Limits::new(100, 100, 11));
        let mut output = String::new();
        context.append(&mut output, "[").unwrap();
        let outcome = project_specified_into(&value.expression, &mut context, &mut output).unwrap();
        context.append(&mut output, "]").unwrap();
        assert_eq!(output, "[calc(0.5)]");
        assert!(!outcome.context_dependent);
        assert_eq!(outcome.scalar_value, Some(0.5));

        let contextual = CssNumberCalculation::try_from_components(
            parse_component_values("calc(1em / 1px)").unwrap(),
        )
        .unwrap();
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let mut output = String::new();
        let outcome =
            project_specified_into(&contextual.expression, &mut context, &mut output).unwrap();
        assert_eq!(output, "calc(1em / 1px)");
        assert!(outcome.context_dependent);
        assert_eq!(outcome.scalar_value, None);
    }

    #[test]
    fn bounded_capture_defers_only_final_output_charging() {
        let value = CssNumberCalculation::try_from_components(
            parse_component_values("calc(1 / 2)").unwrap(),
        )
        .unwrap();
        let mut context = SpecifiedSerializationContext::new(Limits::new(4, 100, 10));
        let (captured, outcome) = capture_specified(&value.expression, &mut context).unwrap();
        assert_eq!(captured, "calc(0.5)");
        assert_eq!(outcome.scalar_value, Some(0.5));

        let mut output = String::new();
        context.append(&mut output, "[").unwrap();
        context.append(&mut output, &captured).unwrap();
        assert_eq!(output, "[calc(0.5)");
        assert_eq!(
            capture_specified(&value.expression, &mut context)
                .unwrap_err()
                .kind(),
            ErrorKind::InputNodeLimit
        );
    }

    #[test]
    fn resolved_compound_dimensions_feed_later_inverse_and_cancellation() {
        for (source, expected) in [
            ("calc((2px * 3px) / (2px * 3px))", "calc(1)"),
            ("calc(1 / (2px * 3px) * 6px * 1px)", "calc(1)"),
            (
                "calc((2px * 3px) / (2em * 3em))",
                "calc(2px * 3px / (2em * 3em))",
            ),
        ] {
            assert_eq!(projected(source), expected, "{source}");
        }
    }

    #[test]
    fn cumulative_budget_counts_replacements_not_only_live_roots() {
        let mut context = SpecifiedSerializationContext::new(Limits::new(100, 3, 100));
        let mut projection = Projection {
            arena: Vec::new(),
            context: &mut context,
        };
        let first = projection
            .value(1.0, Unit::Number, CssNumericType::NUMBER)
            .unwrap();
        let second = projection.negate(first).unwrap();
        let third = projection.negate(second).unwrap();
        assert_eq!(
            projection.negate(third).unwrap_err().kind(),
            ErrorKind::ProjectionNodeLimit
        );
        let mut output = String::new();
        projection
            .serialize(
                third,
                &mut output,
                true,
                &mut NumericEmission::CssComponent(None),
            )
            .unwrap();
        assert_eq!(output, "calc(1)");
    }

    #[test]
    fn scalar_distribution_charges_each_derived_value_and_replacement() {
        let evaluate = |cap| -> Result<String> {
            let mut context = SpecifiedSerializationContext::new(Limits::new(100, cap, 100));
            let mut projection = Projection {
                arena: Vec::new(),
                context: &mut context,
            };
            let length = CssNumericType::dimension(CssNumericDimension::Length);
            let em = projection.value(1.0, Unit::Context("em".into()), length)?;
            let px = projection.value(1.0, Unit::Canonical("px"), length)?;
            let sum = projection.add(Kind::Sum(vec![em, px]), length)?;
            let factor = projection.value(2.0, Unit::Number, CssNumericType::NUMBER)?;
            let product = projection.product(vec![factor, sum], length)?;
            let sign = projection.function(
                Function::Sign,
                vec![Some(product)],
                None,
                CssNumericType::NUMBER,
            )?;
            let mut output = String::new();
            projection.serialize(
                sign,
                &mut output,
                true,
                &mut NumericEmission::CssComponent(None),
            )?;
            Ok(output)
        };
        assert_eq!(
            evaluate(6).unwrap_err().kind(),
            ErrorKind::ProjectionNodeLimit
        );
        assert_eq!(evaluate(20).unwrap(), "sign(2em + 2px)");
    }

    #[test]
    fn flat_and_deep_borrowed_graphs_have_bounded_projection_and_atomic_failure() {
        let source = format!("calc({})", vec!["1"; 4096].join(" + "));
        let value =
            CssNumberCalculation::try_from_components(parse_component_values(&source).unwrap())
                .unwrap();
        let before = value.clone();
        assert_eq!(
            project_specified(&value.expression, Limits::new(100, 10000, 100))
                .unwrap_err()
                .kind(),
            ErrorKind::InputNodeLimit
        );
        assert_eq!(
            project_specified(&value.expression, Limits::new(10000, 100, 100))
                .unwrap_err()
                .kind(),
            ErrorKind::ProjectionNodeLimit
        );
        assert_eq!(
            project_specified(&value.expression, Limits::new(10000, 10000, 100)).unwrap(),
            "calc(4096)"
        );
        assert_eq!(value, before);
        let nested = format!("{}1{}", "calc(".repeat(200), ")".repeat(200));
        assert_eq!(projected(&nested), "calc(1)");
    }

    #[test]
    fn partial_comparisons_keep_first_group_positions_and_symbolic_arguments() {
        for (source, expected) in [
            ("sign(min(1px, 1em))", "sign(min(1px, 1em))"),
            ("sign(min(1em, 1px))", "sign(min(1em, 1px))"),
            (
                "sign(min(2px, 1em - 1px, 1px, 2em))",
                "sign(min(1px, 1em - 1px, 2em))",
            ),
            (
                "sign(max(1px, 1em - 1px, 2px, 2em))",
                "sign(max(2px, 1em - 1px, 2em))",
            ),
            ("clamp(none, sign(1em - 1px), none)", "sign(1em - 1px)"),
        ] {
            assert_eq!(projected(source), expected, "{source}");
        }
    }

    #[test]
    fn nan_poisoning_precedes_symbolic_context_and_compound_dimension_cancellation() {
        for source in [
            "calc(NaN + sign(1em - 1px))",
            "calc(sign(1em - 1px) + NaN)",
            "calc(NaN * sign(1em - 1px))",
            "calc(sign(1em - 1px) * NaN)",
            "calc(infinity + sign(1em - 1px) - infinity)",
            "calc(0px * infinity / 1em)",
            "calc((NaN * 1px * 1px) / (1em * 1em))",
        ] {
            assert_eq!(projected(source), "calc(NaN)", "{source}");
        }
        assert_eq!(
            projected("calc(NaN * sign(1em - 1px) * 1%)"),
            "calc(NaN * 1%)"
        );
    }
}

#[cfg(test)]
mod comparison_capture_tests {
    use super::*;
    use crate::{CssLengthCalculation, CssNumberCalculation, parse_component_values};

    fn expression(source: &str) -> Box<CssCalculationExpression> {
        let components = parse_component_values(source).unwrap();
        if let Ok(value) = CssNumberCalculation::try_from_components(components.clone()) {
            value.expression
        } else {
            CssLengthCalculation::try_from_components(components)
                .unwrap()
                .expression
        }
    }
    fn capture(source: &str) -> NumericComparisonCapture {
        capture_specified_for_comparison(
            &expression(source),
            &mut SpecifiedSerializationContext::new(Limits::default()),
        )
        .unwrap()
        .0
    }

    #[test]
    fn comparison_retains_finite_meaning_before_rounded_text() {
        for (left, right, equal) in [
            ("calc(.12345641)", "calc(.12345642)", false),
            ("calc(1 / 2)", "calc(.5)", true),
            ("calc(0px)", "calc(.0000001px)", false),
            ("calc(0px * -1)", "calc(0px)", true),
            ("calc(.0000001em)", "calc(.0000001px)", false),
            ("calc(1em + .12345641px)", "calc(.12345642px + 1em)", false),
            ("calc(1em + 2px)", "calc(2.0px + 1.0em)", true),
            ("calc(1em - .0000004px)", "calc(1em - .0000003px)", false),
            ("calc(infinity * 1px)", "calc(1px / 0)", true),
            ("calc(NaN * 1px)", "calc(0px / 0)", true),
            ("calc(infinity * 1px)", "calc(-infinity * 1px)", false),
        ] {
            let a = capture(left);
            let b = capture(right);
            assert_eq!(a.same_projected_components(&b), equal, "{left} / {right}");
            assert_eq!(b.same_projected_components(&a), equal);
            for value in [&a, &b] {
                let mut prior = 0;
                for span in &value.finite_coefficients {
                    assert!(
                        prior <= span.start && span.start < span.end && span.end <= value.css.len()
                    );
                    assert!(
                        value.css.is_char_boundary(span.start)
                            && value.css.is_char_boundary(span.end)
                    );
                    assert!(f64::from_bits(span.bits).is_finite());
                    prior = span.end;
                }
                assert!(value.finite_coefficients.len() <= value.css.len());
            }
        }
        assert_eq!(
            capture("calc(.12345641)").as_css(),
            capture("calc(.12345642)").as_css()
        );
    }

    #[test]
    fn synthetic_coefficients_follow_canonical_structure_not_producer_history() {
        for (source, expected_bits) in [
            (
                "calc(0px * -1 + 1em)",
                vec![1.0_f64.to_bits(), 0.0_f64.to_bits(), (-1.0_f64).to_bits()],
            ),
            ("calc(infinity * 1px)", vec![1.0_f64.to_bits()]),
            ("calc(0 * -1)", vec![0.0_f64.to_bits()]),
        ] {
            let value = capture(source);
            assert_eq!(
                value
                    .finite_coefficients
                    .iter()
                    .map(|span| span.bits)
                    .collect::<Vec<_>>(),
                expected_bits
            );
        }
        // Construct canonical nodes through the existing private projector to
        // exercise standalone prefixes without changing parser admission.
        let ty = CssNumericType::NUMBER;
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let mut projection = Projection {
            arena: Vec::new(),
            context: &mut context,
        };
        let symbol = projection.add(Kind::Symbol("symbol".into()), ty).unwrap();
        for (kind, expected, bits) in [
            (
                Kind::Negate(symbol),
                "calc(-1 * symbol)",
                (-1.0_f64).to_bits(),
            ),
            (Kind::Invert(symbol), "calc(1 / symbol)", 1.0_f64.to_bits()),
        ] {
            let root = projection.add(kind, ty).unwrap();
            let mut collector = FiniteCoefficientCollector {
                spans: Vec::new(),
                bound: expected.len(),
            };
            let mut css = String::new();
            projection
                .serialize(
                    root,
                    &mut css,
                    false,
                    &mut NumericEmission::CssComponent(Some(&mut collector)),
                )
                .unwrap();
            assert_eq!(css, expected);
            assert_eq!(collector.spans.len(), 1);
            assert_eq!(collector.spans[0].bits, bits);
        }
        assert!(
            capture("calc(0px * -1 + 1em)")
                .same_projected_components(&capture("calc(1em + 0px * -1)"))
        );
    }

    #[test]
    fn literal_and_synthetic_factors_have_confluent_annotations() {
        let ty = CssNumericType::NUMBER;
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let mut projection = Projection {
            arena: Vec::new(),
            context: &mut context,
        };
        let symbol = projection.add(Kind::Symbol("symbol".into()), ty).unwrap();
        let negative = projection.add(Kind::Negate(symbol), ty).unwrap();
        let inverse = projection.add(Kind::Invert(symbol), ty).unwrap();
        let minus_one = projection.value(-1.0, Unit::Number, ty).unwrap();
        let one = projection.value(1.0, Unit::Number, ty).unwrap();
        let negative_product = projection
            .add(Kind::Product(vec![minus_one, symbol]), ty)
            .unwrap();
        let inverse_product = projection
            .add(Kind::Product(vec![one, inverse]), ty)
            .unwrap();
        let infinity = projection.value(f64::INFINITY, Unit::Number, ty).unwrap();
        let length_ty = CssNumericType::dimension(CssNumericDimension::Length);
        let px = projection
            .value(1.0, Unit::Canonical("px"), length_ty)
            .unwrap();
        let dimensional = projection
            .value(f64::INFINITY, Unit::Canonical("px"), length_ty)
            .unwrap();
        let dimensional_product = projection
            .add(Kind::Product(vec![infinity, px]), length_ty)
            .unwrap();
        let mut render = |root| {
            let mut css = String::new();
            let mut collector = FiniteCoefficientCollector {
                spans: Vec::new(),
                bound: 100,
            };
            projection
                .serialize(
                    root,
                    &mut css,
                    false,
                    &mut NumericEmission::CssComponent(Some(&mut collector)),
                )
                .unwrap();
            NumericComparisonCapture {
                css,
                finite_coefficients: collector.spans,
            }
        };
        for (a, b) in [
            (negative, negative_product),
            (inverse, inverse_product),
            (dimensional, dimensional_product),
        ] {
            let left = render(a);
            let right = render(b);
            assert_eq!(left.as_css(), right.as_css());
            assert!(left.same_projected_components(&right));
        }
    }

    #[test]
    fn selected_capture_matches_plain_text_outcome_and_cumulative_work() {
        let value = expression("calc(1 / 3)");
        let limits = Limits::new(4, 4, 14);
        let mut plain_context = SpecifiedSerializationContext::new(limits);
        let mut selected_context = SpecifiedSerializationContext::new(limits);
        let (plain, a) = capture_specified(&value, &mut plain_context).unwrap();
        let (selected, b) =
            capture_specified_for_comparison(&value, &mut selected_context).unwrap();
        assert_eq!(selected.as_css(), plain);
        assert_eq!(
            a.scalar_value.map(f64::to_bits),
            b.scalar_value.map(f64::to_bits)
        );
        assert_eq!(a.context_dependent, b.context_dependent);
        for context in [&mut plain_context, &mut selected_context] {
            assert_eq!(context.remaining_bytes(), 14);
            assert_eq!(
                context.charge_input(1).unwrap_err().kind(),
                ErrorKind::InputNodeLimit
            );
            assert_eq!(
                context.charge_projection(1).unwrap_err().kind(),
                ErrorKind::ProjectionNodeLimit
            );
            let mut output = String::new();
            context.append(&mut output, "calc(0.333333)").unwrap();
            assert_eq!(context.remaining_bytes(), 0);
            assert_eq!(
                context.append(&mut output, "x").unwrap_err().kind(),
                ErrorKind::ByteLimit
            );
            assert_eq!(output, "calc(0.333333)");
        }
        let mut context = SpecifiedSerializationContext::new(Limits::new(4, 4, 14));
        let mut output = String::new();
        context.append(&mut output, "x").unwrap();
        assert_eq!(
            capture_specified_for_comparison(&value, &mut context)
                .err()
                .unwrap()
                .kind(),
            ErrorKind::ByteLimit
        );
        assert_eq!(output, "x");
        assert_eq!(context.remaining_bytes(), 13);
    }

    #[test]
    fn span_growth_and_rebasing_overflow_are_typed() {
        assert_eq!(span_capacity(0, 1, 8).unwrap(), 1);
        assert_eq!(span_capacity(4, 5, 6).unwrap(), 6);
        assert_eq!(
            span_capacity(0, 1, 0).unwrap_err().kind(),
            ErrorKind::CapacityOverflow
        );
        assert_eq!(
            span_capacity(usize::MAX / 2, usize::MAX, usize::MAX)
                .unwrap_err()
                .kind(),
            ErrorKind::CapacityOverflow
        );
        let span = FiniteCoefficientSpan {
            start: 0,
            end: 1,
            bits: 1.0_f64.to_bits(),
        };
        assert_eq!(
            rebase_span(span, usize::MAX, 1).err().unwrap().kind(),
            ErrorKind::CapacityOverflow
        );
        assert_eq!(
            rebase_span(span, 0, 0).err().unwrap().kind(),
            ErrorKind::CapacityOverflow
        );
        assert_eq!(rebase_span(span, 3, 1).unwrap().end, 4);
        let mut collector = FiniteCoefficientCollector {
            spans: Vec::new(),
            bound: 2,
        };
        collector.append(0, 1, &[span]).unwrap();
        collector.append(1, 1, &[span]).unwrap();
        assert_eq!(collector.spans.len(), 2);
        assert_eq!(
            collector.append(2, 1, &[span]).unwrap_err().kind(),
            ErrorKind::CapacityOverflow
        );
    }

    #[test]
    fn semantic_color_capture_keeps_text_outcome_and_scratch_precedence() {
        let value = expression("calc(1 / 128)");
        let mut color_context = SpecifiedSerializationContext::new(Limits::new(4, 4, 15));
        let (color, color_outcome) = capture_color_specified_scaled(
            &value,
            NumericProjectionScale::Identity,
            &mut color_context,
        )
        .unwrap();
        assert_eq!(color, "calc(0.0078125)");
        assert_eq!(color_outcome.scalar_value, Some(1.0 / 128.0));
        let mut generic_context = SpecifiedSerializationContext::new(Limits::new(4, 4, 14));
        let (generic, generic_outcome) = capture_specified(&value, &mut generic_context).unwrap();
        assert_eq!(generic, "calc(0.007813)");
        assert_eq!(
            generic_outcome.scalar_value.map(f64::to_bits),
            color_outcome.scalar_value.map(f64::to_bits)
        );
        assert_eq!(color_context.remaining_bytes(), 15);
        assert_eq!(
            color_context.charge_input(1).unwrap_err().kind(),
            ErrorKind::InputNodeLimit
        );
        assert_eq!(
            color_context.charge_projection(1).unwrap_err().kind(),
            ErrorKind::ProjectionNodeLimit
        );
        let mut short = SpecifiedSerializationContext::new(Limits::new(4, 4, 14));
        assert_eq!(
            capture_color_specified_scaled(&value, NumericProjectionScale::Identity, &mut short)
                .unwrap_err()
                .kind(),
            ErrorKind::ByteLimit
        );
        for emission in [
            NumericEmission::ColorCalculation,
            NumericEmission::CssComponent(None),
        ] {
            let scalar = Scalar {
                value: 1.0 / 128.0,
                unit: Unit::Number,
            };
            assert_eq!(
                emission
                    .scalar_piece(&scalar, true, false, 20)
                    .unwrap()
                    .count,
                0
            );
        }
    }
}

#[cfg(test)]
mod scaled_capture_tests {
    use super::*;
    use crate::{CssNumberCalculation, CssPercentageCalculation, parse_component_values};

    fn number(source: &str) -> CssNumberCalculation {
        CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap()
    }

    #[test]
    fn scaled_percentage_rounds_text_without_rounding_the_scalar() {
        let value = CssPercentageCalculation::try_from_components(
            parse_component_values("calc(.78125%)").unwrap(),
        )
        .unwrap();
        let before = value.clone();
        // A calc group and percentage leaf visit two input nodes. Projection
        // allocates the leaf, dimensional factor, inverse, and resolved product.
        let mut context = SpecifiedSerializationContext::new(Limits::new(2, 4, 14));
        let (text, outcome) = capture_specified_scaled(
            &value.expression,
            NumericProjectionScale::PercentageToNumber {
                numerator: 1,
                denominator: 100,
            },
            &mut context,
        )
        .unwrap();
        assert_eq!(text, "calc(0.007813)");
        assert_eq!(
            outcome.scalar_value.unwrap().to_bits(),
            (1.0_f64 / 128.0).to_bits()
        );
        assert!(!outcome.context_dependent);
        assert_eq!(context.remaining_bytes(), 14);
        assert_eq!(
            context.charge_input(1).unwrap_err().kind(),
            ErrorKind::InputNodeLimit
        );
        assert_eq!(
            context.charge_projection(1).unwrap_err().kind(),
            ErrorKind::ProjectionNodeLimit
        );
        assert_eq!(value, before);
    }

    #[test]
    fn scaled_number_uses_actual_binary_coefficient_and_preserves_authored_graph() {
        for (source, scale, expected, scalar) in [
            (
                "calc(1 / 256)",
                NumericProjectionScale::Number {
                    numerator: 2,
                    denominator: 1,
                },
                "calc(0.007813)",
                1.0_f64 / 128.0,
            ),
            (
                "calc(5e-7)",
                NumericProjectionScale::Identity,
                "calc(0)",
                5e-7_f64,
            ),
            (
                "calc(-1 / 128)",
                NumericProjectionScale::Identity,
                "calc(-0.007813)",
                -1.0_f64 / 128.0,
            ),
        ] {
            let value = number(source);
            let before = value.clone();
            let (text, outcome) = capture_specified_scaled(
                &value.expression,
                scale,
                &mut SpecifiedSerializationContext::new(Limits::default()),
            )
            .unwrap();
            assert_eq!(text, expected);
            assert_eq!(outcome.scalar_value.unwrap().to_bits(), scalar.to_bits());
            assert!(!outcome.context_dependent);
            assert_eq!(value, before);
        }
    }

    #[test]
    fn identity_and_color_captures_preserve_traversal_but_select_coefficient_text() {
        let value = number("calc(1 / 128)");
        // The calc group, product, and two leaves are four inputs. The
        // projected leaves, scalar inverse, and folded product are four nodes.
        for color in [false, true] {
            let mut context = SpecifiedSerializationContext::new(Limits::new(4, 4, 15));
            let (text, outcome) = if color {
                capture_color_specified_scaled(
                    &value.expression,
                    NumericProjectionScale::Identity,
                    &mut context,
                )
            } else {
                capture_specified(&value.expression, &mut context)
            }
            .unwrap();
            assert_eq!(
                text,
                if color {
                    "calc(0.0078125)"
                } else {
                    "calc(0.007813)"
                }
            );
            assert_eq!(
                outcome.scalar_value.unwrap().to_bits(),
                (1.0_f64 / 128.0).to_bits()
            );
            assert_eq!(
                context.charge_input(1).unwrap_err().kind(),
                ErrorKind::InputNodeLimit
            );
            assert_eq!(
                context.charge_projection(1).unwrap_err().kind(),
                ErrorKind::ProjectionNodeLimit
            );
        }
    }

    #[test]
    fn scaled_capture_bounds_scratch_and_charges_accepted_text_once() {
        let value = number("calc(1 / 256)");
        let scale = NumericProjectionScale::Number {
            numerator: 2,
            denominator: 1,
        };
        // Scaling adds a factor and folded product to the four-node projection.
        let mut context = SpecifiedSerializationContext::new(Limits::new(4, 6, 14));
        let (text, _) = capture_specified_scaled(&value.expression, scale, &mut context).unwrap();
        assert_eq!(text, "calc(0.007813)");
        assert_eq!(context.remaining_bytes(), 14);
        let mut output = String::new();
        context.append(&mut output, &text).unwrap();
        assert_eq!(context.remaining_bytes(), 0);
        assert_eq!(
            context.append(&mut output, "!").unwrap_err().kind(),
            ErrorKind::ByteLimit
        );
        assert_eq!(output, text);
        let before = value.clone();
        let mut short = SpecifiedSerializationContext::new(Limits::new(4, 6, 13));
        assert_eq!(
            capture_specified_scaled(&value.expression, scale, &mut short)
                .unwrap_err()
                .kind(),
            ErrorKind::ByteLimit
        );
        assert_eq!(short.remaining_bytes(), 13);
        assert_eq!(value, before);
    }

    #[test]
    fn scaled_sibling_captures_keep_input_and_projection_work_cumulative() {
        let value = number("calc(1 / 256)");
        let before = value.clone();
        let scale = NumericProjectionScale::Number {
            numerator: 2,
            denominator: 1,
        };
        for (limits, expected) in [
            (Limits::new(4, 100, 14), ErrorKind::InputNodeLimit),
            (Limits::new(100, 6, 14), ErrorKind::ProjectionNodeLimit),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            assert_eq!(
                capture_specified_scaled(&value.expression, scale, &mut context)
                    .unwrap()
                    .0,
                "calc(0.007813)"
            );
            assert_eq!(
                capture_specified_scaled(&value.expression, scale, &mut context)
                    .unwrap_err()
                    .kind(),
                expected
            );
            assert_eq!(context.remaining_bytes(), 14);
            assert_eq!(value, before);
        }
    }

    #[test]
    fn scaled_capture_retains_nonfinite_signed_zero_and_contextual_operands() {
        for (source, expected, scalar) in [
            ("calc(infinity)", "calc(infinity)", f64::INFINITY),
            ("calc(-infinity)", "calc(-infinity)", f64::NEG_INFINITY),
            ("calc(0 * -1)", "calc(0)", -0.0_f64),
        ] {
            let (text, outcome) = capture_specified_scaled(
                &number(source).expression,
                NumericProjectionScale::Identity,
                &mut SpecifiedSerializationContext::new(Limits::default()),
            )
            .unwrap();
            assert_eq!(text, expected);
            assert_eq!(outcome.scalar_value.unwrap().to_bits(), scalar.to_bits());
            assert!(!outcome.context_dependent);
        }
        let (text, outcome) = capture_specified_scaled(
            &number("calc(NaN)").expression,
            NumericProjectionScale::Identity,
            &mut SpecifiedSerializationContext::new(Limits::default()),
        )
        .unwrap();
        assert_eq!(text, "calc(NaN)");
        assert!(outcome.scalar_value.unwrap().is_nan());
        assert!(!outcome.context_dependent);
        let value = number("calc(1em / 1px)");
        let before = value.clone();
        let (text, outcome) = capture_specified_scaled(
            &value.expression,
            NumericProjectionScale::Identity,
            &mut SpecifiedSerializationContext::new(Limits::default()),
        )
        .unwrap();
        assert_eq!(text, "calc(1em / 1px)");
        assert!(outcome.context_dependent);
        assert_eq!(outcome.scalar_value, None);
        assert_eq!(value, before);
    }
}
