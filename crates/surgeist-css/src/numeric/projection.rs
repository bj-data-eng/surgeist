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
            Kind::Scalar(value) if value.unit.resolved() || value.value.is_nan() => {
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
        let mut other = Vec::new();
        let mut scalars: BTreeMap<Unit, f64> = BTreeMap::new();
        // Pop backwards, then restore original order for symbolic terms.
        while let Some(id) = pending.pop() {
            if matches!(self.arena[id].kind, Kind::Sum(_)) {
                let Kind::Sum(children) =
                    std::mem::replace(&mut self.arena[id].kind, Kind::Consumed)
                else {
                    unreachable!()
                };
                pending.extend(children);
            } else if let Some(scalar) = self.scalar(id) {
                scalars
                    .entry(scalar.unit.clone())
                    .and_modify(|value| *value += scalar.value)
                    .or_insert(scalar.value);
            } else {
                other.push(id);
            }
        }
        other.reverse();
        if ty.simple() && scalars.values().any(|value| value.is_nan()) {
            return self.value(f64::NAN, Unit::from_type(ty), ty);
        }
        let mut combined = Vec::new();
        for (unit, value) in scalars {
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
        if ty.simple() {
            let resolved = self.resolved_terms(&flat, true);
            if let Some(value) = resolved {
                return self.value(value, Unit::from_type(ty), ty);
            }
        }
        let mut number = None;
        let mut other = Vec::new();
        for id in flat {
            if let Some(value) = self.scalar(id)
                && value.unit == Unit::Number
            {
                number = Some(number.unwrap_or(1.0) * value.value);
            } else {
                other.push(id);
            }
        }
        if let Some(number) = number {
            if other.len() == 1 {
                let child = other[0];
                if let Some(value) = self.scalar(child).cloned() {
                    return self.value(number * value.value, value.unit, ty);
                }
                if let Kind::Sum(children) = &self.arena[child].kind
                    && children.iter().all(|&id| self.scalar(id).is_some())
                {
                    let children = children.clone();
                    let mut distributed = Vec::new();
                    for id in children {
                        let value = self.scalar(id).expect("checked scalar child").clone();
                        distributed.push(self.value(
                            number * value.value,
                            value.unit,
                            self.arena[id].ty,
                        )?);
                    }
                    return self.sum(distributed, ty);
                }
            }
            other.insert(0, self.value(number, Unit::Number, CssNumericType::NUMBER)?);
        }
        if other.len() == 1 {
            return Ok(other[0]);
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
            let resolved = values.iter().flatten().all(|v| v.unit.resolved());
            let same_unit = values
                .iter()
                .flatten()
                .map(|v| &v.unit)
                .all(|unit| Some(unit) == values.iter().flatten().next().map(|v| &v.unit));
            if resolved
                || same_unit && matches!(function, Function::Min | Function::Max | Function::Clamp)
            {
                let output = math::evaluate(function, &values, strategy);
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
                if let Some(scalar) = self.scalar(id) {
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

pub(crate) fn capture_specified_scaled(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
) -> Result<(String, NumericProjectionOutcome)> {
    let mut output = String::new();
    let outcome = project_specified_impl(expression, scale, context, &mut output, false)?;
    Ok((output, outcome))
}

fn project_specified_impl(
    expression: &CssCalculationExpression,
    scale: NumericProjectionScale,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    charge_output: bool,
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
                        let value = lexical_value(number.representation()) * factor;
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
    projection.serialize(root, output, charge_output)?;
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
        } => (numerator, denominator, Unit::Percentage, number),
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
}

impl Projection<'_> {
    fn serialize(&mut self, root: Id, output: &mut String, charge_output: bool) -> Result<()> {
        let mut work = Vec::new();
        if matches!(
            self.arena[root].kind,
            Kind::Function { .. } | Kind::Symbol(_)
        ) {
            work.push(Output::Node(root, Position::Root));
        } else {
            work.push(Output::Text(")".into()));
            work.push(Output::Node(root, Position::Root));
            work.push(Output::Text("calc(".into()));
        }
        while let Some(item) = work.pop() {
            let (id, position) = match item {
                Output::Text(text) => {
                    if charge_output {
                        self.context.append(output, &text)?;
                    } else {
                        self.context.append_temporary(output, &text)?;
                    }
                    continue;
                }
                Output::Node(id, position) => (id, position),
            };
            let mut next = Vec::new();
            match &self.arena[id].kind {
                Kind::Scalar(value) => next.push(Output::Text(scalar_text(
                    value,
                    matches!(position, Position::Root),
                ))),
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
                            let mut value = self.scalar(child).expect("negative scalar").clone();
                            value.value = -value.value;
                            next.push(Output::Text(scalar_text(&value, false)));
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
                    next.push(Output::Text(
                        if matches!(self.arena[id].kind, Kind::Negate(_)) {
                            "-1 * "
                        } else {
                            "1 / "
                        }
                        .into(),
                    ));
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

fn scalar_text(scalar: &Scalar, root: bool) -> String {
    let unit = scalar.unit.name();
    if scalar.value.is_finite() {
        if scalar.value == 0.0 && scalar.value.is_sign_negative() && !root {
            return format!("(0{unit} * -1)");
        }
        return format!(
            "{}{unit}",
            crate::specified_serialization::format_binary64(scalar.value)
        );
    }
    let keyword = if scalar.value.is_nan() {
        "NaN"
    } else if scalar.value.is_sign_negative() {
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
        projection.serialize(third, &mut output, true).unwrap();
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
            projection.serialize(sign, &mut output, true)?;
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
