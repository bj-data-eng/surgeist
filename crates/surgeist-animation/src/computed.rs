//! Computed numeric values and whole-property combination.
//!
//! Dimensions use canonical units (px, deg, s, Hz and dppx). Percentages remain
//! ratios, and positions are x/y offsets from the top-left corner. Callers own
//! authored CSS, contextual evaluation and final property range clamping.
//!
//! A [`MathSource`] identifies one caller-owned computed calculation in one
//! immutable [`MathContext`]. The caller retains the source-to-calculation mapping
//! while unresolved samples are needed. Animation retains identity and typed
//! combination operations, without parsing, evaluating or serializing CSS.
//!
//! ```
//! use surgeist_animation::computed::{ComputedNumeric, MathContext, NumericType};
//! use surgeist_animation::interpolation::InterpolationProgress;
//!
//! let context = MathContext::new();
//! let source = context.source(NumericType::Number);
//! let first = ComputedNumeric::source(source.clone());
//! let second = ComputedNumeric::number(50.0)?;
//! let result = first.interpolate(&second, InterpolationProgress::new(0.25)?)?;
//! assert_eq!(result.numeric_type(), NumericType::Number);
//! assert_eq!(source.context(), &context);
//! // The caller retains the source's computed calculation and context, then
//! // evaluates the returned postorder plan when that context is available.
//! assert!(result.math_nodes().is_some());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::interpolation::{DiscreteValue, InterpolationProgress, checked_interpolate};

/// A computed dimension's canonical unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionKind {
    /// CSS pixels.
    Length,
    /// Degrees.
    Angle,
    /// Seconds.
    Time,
    /// Hertz.
    Frequency,
    /// Device pixels per CSS pixel.
    Resolution,
}

/// A computed numeric result type, including its percentage interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumericType {
    Number,
    Integer,
    Percentage,
    Dimension(DimensionKind),
    DimensionPercentage(DimensionKind),
}

/// The operation that produced a computed sample or diagnostic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedOperation {
    Interpolate(InterpolationProgress),
    Add,
    Accumulate,
}

/// A checked computed-value operation's failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputedErrorKind {
    NonFiniteInput,
    FractionalInteger,
    NegativeRatioComponent,
    DegenerateRatio,
    IncompatibleTypes,
    EmptyComponents,
    NonFiniteResult,
    RatioUnderflow,
    Capacity,
}

/// A semantic computed-value error, optionally identifying a property component.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedError {
    kind: ComputedErrorKind,
    operation: Option<ComputedOperation>,
    component: Option<usize>,
}

impl ComputedError {
    fn new(kind: ComputedErrorKind) -> Self {
        Self {
            kind,
            operation: None,
            component: None,
        }
    }

    fn during(mut self, operation: ComputedOperation) -> Self {
        self.operation = Some(operation);
        self
    }

    fn at(mut self, component: usize) -> Self {
        self.component = Some(component);
        self
    }

    pub const fn kind(&self) -> ComputedErrorKind {
        self.kind
    }
    pub const fn operation(&self) -> Option<ComputedOperation> {
        self.operation
    }
    pub const fn component(&self) -> Option<usize> {
        self.component
    }
}

impl fmt::Display for ComputedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "computed value {:?}", self.kind)?;
        if let Some(operation) = self.operation {
            write!(f, " during {operation:?}")?;
        }
        if let Some(component) = self.component {
            write!(f, " at component {component}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ComputedError {}

/// A finite binary64 integer; no property-specific range is imposed.
///
/// The supported integers are the integral finite binary64 values. Conversion
/// from exact authored integers belongs to the caller's computed-value lowering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComputedInteger {
    value: f64,
}

impl ComputedInteger {
    pub fn new(value: f64) -> Result<Self, ComputedError> {
        finite(value)?;
        if value.trunc() != value {
            return Err(ComputedError::new(ComputedErrorKind::FractionalInteger));
        }
        Ok(Self { value })
    }

    pub const fn value(self) -> f64 {
        self.value
    }

    pub fn interpolate(
        self,
        other: Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ComputedError> {
        let value = scalar(
            self.value,
            other.value,
            ComputedOperation::Interpolate(progress),
        )?;
        // Adding .5 first can round the predecessor of .5 up to 1. Compare the
        // fractional part instead. Integral large magnitudes need no increment.
        let lower = value.floor();
        let rounded = if value - lower < 0.5 {
            lower
        } else {
            lower + 1.0
        };
        Self::new(rounded)
    }

    pub fn add(&self, other: &Self) -> Result<Self, ComputedError> {
        Self::new(scalar(self.value, other.value, ComputedOperation::Add)?)
    }

    pub fn accumulate(&self, other: &Self) -> Result<Self, ComputedError> {
        Self::new(scalar(
            self.value,
            other.value,
            ComputedOperation::Accumulate,
        )?)
    }
}

/// A finite computed dimension in its kind's canonical unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComputedDimension {
    kind: DimensionKind,
    value: f64,
}

impl ComputedDimension {
    pub fn new(kind: DimensionKind, value: f64) -> Result<Self, ComputedError> {
        Ok(Self {
            kind,
            value: finite(value)?,
        })
    }
    pub const fn kind(self) -> DimensionKind {
        self.kind
    }
    pub const fn value(self) -> f64 {
        self.value
    }
}

/// Independent canonical dimension and percentage terms of a computed mix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DimensionPercentage {
    dimension: ComputedDimension,
    percentage: f64,
}

impl DimensionPercentage {
    pub fn new(dimension: ComputedDimension, percentage: f64) -> Result<Self, ComputedError> {
        Ok(Self {
            dimension,
            percentage: finite(percentage)?,
        })
    }
    pub const fn dimension(self) -> ComputedDimension {
        self.dimension
    }
    pub const fn percentage(self) -> f64 {
        self.percentage
    }
}

/// An immutable identity for all context affecting caller-owned calculations.
#[derive(Debug, Clone)]
pub struct MathContext {
    identity: Arc<()>,
}

impl MathContext {
    /// Creates a distinct context; clones preserve its identity.
    pub fn new() -> Self {
        Self {
            identity: Arc::new(()),
        }
    }
    /// Creates a distinct source in this context with the supplied computed type.
    pub fn source(&self, numeric_type: NumericType) -> MathSource {
        MathSource {
            identity: Arc::new(SourceIdentity {
                context: self.clone(),
                numeric_type,
            }),
        }
    }
}

impl Default for MathContext {
    fn default() -> Self {
        Self::new()
    }
}
impl PartialEq for MathContext {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}
impl Eq for MathContext {}
impl Hash for MathContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::ptr::hash(Arc::as_ptr(&self.identity), state);
    }
}

#[derive(Debug)]
struct SourceIdentity {
    context: MathContext,
    numeric_type: NumericType,
}

/// Identity of one contextual computed numeric calculation retained by the caller.
#[derive(Debug, Clone)]
pub struct MathSource {
    identity: Arc<SourceIdentity>,
}

impl MathSource {
    pub fn numeric_type(&self) -> NumericType {
        self.identity.numeric_type
    }
    pub fn context(&self) -> &MathContext {
        &self.identity.context
    }
}
impl PartialEq for MathSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}
impl Eq for MathSource {}
impl Hash for MathSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::ptr::hash(Arc::as_ptr(&self.identity), state);
    }
}

/// A literal numeric operand in canonical computed units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericLiteral {
    Number(f64),
    Integer(ComputedInteger),
    Percentage(f64),
    Dimension(ComputedDimension),
    Mix(DimensionPercentage),
}

impl NumericLiteral {
    fn validate(self) -> Result<(), ComputedError> {
        match self {
            Self::Number(value) | Self::Percentage(value) => {
                finite(value)?;
            }
            Self::Integer(_) | Self::Dimension(_) | Self::Mix(_) => {}
        }
        Ok(())
    }
    pub const fn numeric_type(self) -> NumericType {
        match self {
            Self::Number(_) => NumericType::Number,
            Self::Integer(_) => NumericType::Integer,
            Self::Percentage(_) => NumericType::Percentage,
            Self::Dimension(value) => NumericType::Dimension(value.kind),
            Self::Mix(value) => NumericType::DimensionPercentage(value.dimension.kind),
        }
    }
}

/// One immutable postorder operation in a symbolic numeric plan.
///
/// Operand indices always precede this node. Interpolation of integer results
/// includes nearest-integer rounding with ties toward positive infinity.
#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    Literal(NumericLiteral),
    Source(MathSource),
    Combine {
        operation: ComputedOperation,
        numeric_type: NumericType,
        first: usize,
        second: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct MathPlan {
    numeric_type: NumericType,
    nodes: Arc<[MathNode]>,
}

/// A validated computed literal or unresolved symbolic combination plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedNumeric {
    repr: NumericRepr,
}

#[derive(Debug, Clone, PartialEq)]
enum NumericRepr {
    Literal(NumericLiteral),
    Plan(MathPlan),
}

impl ComputedNumeric {
    pub fn literal(value: NumericLiteral) -> Result<Self, ComputedError> {
        value.validate()?;
        if let NumericLiteral::Mix(value) = value {
            return Ok(Self::mix(value));
        }
        Ok(Self {
            repr: NumericRepr::Literal(value),
        })
    }
    pub fn number(value: f64) -> Result<Self, ComputedError> {
        Self::literal(NumericLiteral::Number(value))
    }
    pub fn percentage(value: f64) -> Result<Self, ComputedError> {
        Self::literal(NumericLiteral::Percentage(value))
    }
    pub const fn integer(value: ComputedInteger) -> Self {
        Self {
            repr: NumericRepr::Literal(NumericLiteral::Integer(value)),
        }
    }
    pub const fn dimension(value: ComputedDimension) -> Self {
        Self {
            repr: NumericRepr::Literal(NumericLiteral::Dimension(value)),
        }
    }
    pub const fn mix(value: DimensionPercentage) -> Self {
        Self {
            repr: NumericRepr::Literal(if value.percentage == 0.0 {
                NumericLiteral::Dimension(value.dimension)
            } else if value.dimension.value == 0.0 {
                NumericLiteral::Percentage(value.percentage)
            } else {
                NumericLiteral::Mix(value)
            }),
        }
    }
    pub fn source(source: MathSource) -> Self {
        Self {
            repr: NumericRepr::Plan(MathPlan {
                numeric_type: source.numeric_type(),
                nodes: Arc::from([MathNode::Source(source)]),
            }),
        }
    }
    pub fn numeric_type(&self) -> NumericType {
        match &self.repr {
            NumericRepr::Literal(value) => value.numeric_type(),
            NumericRepr::Plan(plan) => plan.numeric_type,
        }
    }
    pub const fn as_literal(&self) -> Option<NumericLiteral> {
        match self.repr {
            NumericRepr::Literal(value) => Some(value),
            NumericRepr::Plan(_) => None,
        }
    }
    /// Borrows postorder symbolic nodes; the final node is the result.
    pub fn math_nodes(&self) -> Option<&[MathNode]> {
        match &self.repr {
            NumericRepr::Literal(_) => None,
            NumericRepr::Plan(plan) => Some(&plan.nodes),
        }
    }
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Interpolate(progress))
    }
    pub fn add(&self, other: &Self) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Add)
    }
    pub fn accumulate(&self, other: &Self) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Accumulate)
    }

    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, ComputedError> {
        let numeric_type =
            compatible_type(self.numeric_type(), other.numeric_type()).ok_or_else(|| {
                ComputedError::new(ComputedErrorKind::IncompatibleTypes).during(operation)
            })?;
        if let ComputedOperation::Interpolate(progress) = operation {
            if progress.value() == 0.0 {
                return Ok(self.clone());
            }
            if progress.value() == 1.0 {
                return Ok(other.clone());
            }
        }
        match (&self.repr, &other.repr) {
            (NumericRepr::Literal(first), NumericRepr::Literal(second)) => {
                combine_literals(*first, *second, numeric_type, operation)
            }
            _ => self.symbolic(other, numeric_type, operation),
        }
    }

    fn symbolic(
        &self,
        other: &Self,
        numeric_type: NumericType,
        operation: ComputedOperation,
    ) -> Result<Self, ComputedError> {
        let first_len = self.node_count();
        let second_len = other.node_count();
        let length = first_len
            .checked_add(second_len)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| ComputedError::new(ComputedErrorKind::Capacity).during(operation))?;
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(length)
            .map_err(|_| ComputedError::new(ComputedErrorKind::Capacity).during(operation))?;
        self.append_nodes(&mut nodes, 0)?;
        other.append_nodes(&mut nodes, first_len)?;
        // Every validated input contains at least its literal/source root.
        let first = first_len
            .checked_sub(1)
            .ok_or_else(|| ComputedError::new(ComputedErrorKind::Capacity))?;
        let second = length
            .checked_sub(2)
            .ok_or_else(|| ComputedError::new(ComputedErrorKind::Capacity))?;
        nodes.push(MathNode::Combine {
            operation,
            numeric_type,
            first,
            second,
        });
        Ok(Self {
            repr: NumericRepr::Plan(MathPlan {
                numeric_type,
                nodes: nodes.into(),
            }),
        })
    }
    fn node_count(&self) -> usize {
        match &self.repr {
            NumericRepr::Literal(_) => 1,
            NumericRepr::Plan(plan) => plan.nodes.len(),
        }
    }
    fn append_nodes(&self, nodes: &mut Vec<MathNode>, offset: usize) -> Result<(), ComputedError> {
        match &self.repr {
            NumericRepr::Literal(value) => nodes.push(MathNode::Literal(*value)),
            NumericRepr::Plan(plan) => {
                for node in plan.nodes.iter() {
                    let node = match node {
                        MathNode::Combine {
                            operation,
                            numeric_type,
                            first,
                            second,
                        } => MathNode::Combine {
                            operation: *operation,
                            numeric_type: *numeric_type,
                            first: first
                                .checked_add(offset)
                                .ok_or_else(|| ComputedError::new(ComputedErrorKind::Capacity))?,
                            second: second
                                .checked_add(offset)
                                .ok_or_else(|| ComputedError::new(ComputedErrorKind::Capacity))?,
                        },
                        node => node.clone(),
                    };
                    nodes.push(node);
                }
            }
        }
        Ok(())
    }
}

fn compatible_type(first: NumericType, second: NumericType) -> Option<NumericType> {
    use NumericType::{Dimension, DimensionPercentage, Percentage};
    if first == second {
        return Some(first);
    }
    match (first, second) {
        (Dimension(kind) | DimensionPercentage(kind), Percentage)
        | (Percentage, Dimension(kind) | DimensionPercentage(kind)) => {
            Some(DimensionPercentage(kind))
        }
        (
            Dimension(first) | DimensionPercentage(first),
            Dimension(second) | DimensionPercentage(second),
        ) if first == second => Some(DimensionPercentage(first)),
        _ => None,
    }
}

fn combine_literals(
    first: NumericLiteral,
    second: NumericLiteral,
    numeric_type: NumericType,
    operation: ComputedOperation,
) -> Result<ComputedNumeric, ComputedError> {
    let literal = match (first, second) {
        (NumericLiteral::Number(first), NumericLiteral::Number(second)) => {
            NumericLiteral::Number(scalar(first, second, operation)?)
        }
        (NumericLiteral::Integer(first), NumericLiteral::Integer(second)) => {
            let value = match operation {
                ComputedOperation::Interpolate(progress) => first.interpolate(second, progress)?,
                ComputedOperation::Add => first.add(&second)?,
                ComputedOperation::Accumulate => first.accumulate(&second)?,
            };
            NumericLiteral::Integer(value)
        }
        (NumericLiteral::Percentage(first), NumericLiteral::Percentage(second)) => {
            NumericLiteral::Percentage(scalar(first, second, operation)?)
        }
        _ => {
            let kind = match numeric_type {
                NumericType::Dimension(kind) | NumericType::DimensionPercentage(kind) => kind,
                _ => {
                    return Err(
                        ComputedError::new(ComputedErrorKind::IncompatibleTypes).during(operation)
                    );
                }
            };
            let (first_dimension, first_percentage) = literal_terms(first)?;
            let (second_dimension, second_percentage) = literal_terms(second)?;
            let dimension = ComputedDimension::new(
                kind,
                scalar(first_dimension, second_dimension, operation)?,
            )?;
            let percentage = scalar(first_percentage, second_percentage, operation)?;
            if percentage == 0.0 {
                NumericLiteral::Dimension(dimension)
            } else if dimension.value == 0.0 {
                NumericLiteral::Percentage(percentage)
            } else {
                NumericLiteral::Mix(DimensionPercentage::new(dimension, percentage)?)
            }
        }
    };
    ComputedNumeric::literal(literal)
}

fn literal_terms(value: NumericLiteral) -> Result<(f64, f64), ComputedError> {
    match value {
        NumericLiteral::Dimension(value) => Ok((value.value, 0.0)),
        NumericLiteral::Percentage(value) => Ok((0.0, value)),
        NumericLiteral::Mix(value) => Ok((value.dimension.value, value.percentage)),
        _ => Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes)),
    }
}

/// A computed ratio preserving both nonnegative extended-real components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComputedRatio {
    numerator: f64,
    denominator: f64,
}

impl ComputedRatio {
    pub fn new(numerator: f64, denominator: f64) -> Result<Self, ComputedError> {
        for value in [numerator, denominator] {
            if value.is_nan() {
                return Err(ComputedError::new(ComputedErrorKind::NonFiniteInput));
            }
            if value < 0.0 {
                return Err(ComputedError::new(
                    ComputedErrorKind::NegativeRatioComponent,
                ));
            }
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
    pub const fn numerator(self) -> f64 {
        self.numerator
    }
    pub const fn denominator(self) -> f64 {
        self.denominator
    }
    pub fn is_degenerate(self) -> bool {
        self.numerator == 0.0
            || self.denominator == 0.0
            || self.numerator.is_infinite()
            || self.denominator.is_infinite()
    }
    /// Compares quotients, returning `None` for 0/0 or infinity/infinity.
    /// Component equality remains distinct from quotient comparison.
    pub fn compare(self, other: Self) -> Option<Ordering> {
        let first = quotient_class(self)?;
        let second = quotient_class(other)?;
        match first.cmp(&second) {
            Ordering::Equal if first == QuotientClass::Finite => {
                let (a, ae) = significand(self.numerator);
                let (b, be) = significand(other.denominator);
                let (c, ce) = significand(other.numerator);
                let (d, de) = significand(self.denominator);
                // Each binary64 significand has at most 53 bits. The exact
                // cross-products therefore fit in 106 bits of a u128.
                Some(compare_products(
                    u128::from(a) * u128::from(b),
                    ae + be,
                    u128::from(c) * u128::from(d),
                    ce + de,
                ))
            }
            order => Some(order),
        }
    }
    pub fn interpolate(
        self,
        other: Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ComputedError> {
        let operation = ComputedOperation::Interpolate(progress);
        if self.is_degenerate() || other.is_degenerate() {
            return Err(ComputedError::new(ComputedErrorKind::DegenerateRatio).during(operation));
        }
        if progress.value() == 0.0 {
            return Ok(self);
        }
        if progress.value() == 1.0 {
            return Ok(other);
        }
        let first = self.numerator.ln() - self.denominator.ln();
        let second = other.numerator.ln() - other.denominator.ln();
        let value = scalar(first, second, operation)?.exp();
        if !value.is_finite() {
            return Err(ComputedError::new(ComputedErrorKind::NonFiniteResult).during(operation));
        }
        if value == 0.0 {
            return Err(ComputedError::new(ComputedErrorKind::RatioUnderflow).during(operation));
        }
        Self::new(value, 1.0)
    }
    /// Ratios are not additive: composition selects the second operand.
    pub const fn add(self, other: Self) -> Self {
        other
    }
    pub const fn accumulate(self, other: Self) -> Self {
        other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum QuotientClass {
    Zero,
    Finite,
    Infinite,
}

fn quotient_class(ratio: ComputedRatio) -> Option<QuotientClass> {
    if (ratio.numerator == 0.0 && ratio.denominator == 0.0)
        || (ratio.numerator.is_infinite() && ratio.denominator.is_infinite())
    {
        None
    } else if ratio.numerator == 0.0 || ratio.denominator.is_infinite() {
        Some(QuotientClass::Zero)
    } else if ratio.denominator == 0.0 || ratio.numerator.is_infinite() {
        Some(QuotientClass::Infinite)
    } else {
        Some(QuotientClass::Finite)
    }
}

fn significand(value: f64) -> (u64, i32) {
    // Binary64 has 52 stored fraction bits and exponent bias 1023. Normal
    // significands include the hidden bit and use bias + 52 = 1075; subnormals
    // have no hidden bit and use the fixed least-bit exponent -1074.
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1075)
    }
}

fn compare_products(
    first: u128,
    first_exponent: i32,
    second: u128,
    second_exponent: i32,
) -> Ordering {
    // Normalizing both products to the high bit permits an exact lexicographic
    // exponent/significand comparison. Shifts stay strictly below 128, including
    // a defensive zero operand; finite positive ratios supply nonzero products.
    let first_shift = first.leading_zeros().min(127);
    let second_shift = second.leading_zeros().min(127);
    let first_scale = first_exponent - first_shift as i32;
    let second_scale = second_exponent - second_shift as i32;
    first_scale
        .cmp(&second_scale)
        .then_with(|| (first << first_shift).cmp(&(second << second_shift)))
}

/// Computed x/y offsets from the top-left corner, retaining contextual math.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedPosition {
    x: ComputedNumeric,
    y: ComputedNumeric,
}

impl ComputedPosition {
    pub fn new(x: ComputedNumeric, y: ComputedNumeric) -> Result<Self, ComputedError> {
        for value in [&x, &y] {
            match value.numeric_type() {
                NumericType::Dimension(DimensionKind::Length)
                | NumericType::DimensionPercentage(DimensionKind::Length)
                | NumericType::Percentage => {}
                _ => return Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes)),
            }
        }
        Ok(Self { x, y })
    }
    pub const fn x(&self) -> &ComputedNumeric {
        &self.x
    }
    pub const fn y(&self) -> &ComputedNumeric {
        &self.y
    }
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Interpolate(progress))
    }
    pub fn add(&self, other: &Self) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Add)
    }
    pub fn accumulate(&self, other: &Self) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Accumulate)
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, ComputedError> {
        Self::new(
            self.x.combine(&other.x, operation)?,
            self.y.combine(&other.y, operation)?,
        )
    }
}

/// One component of a normalized computed property value.
#[derive(Debug, Clone, PartialEq)]
pub enum ComputedComponent {
    Numeric(ComputedNumeric),
    Ratio(ComputedRatio),
    Position(ComputedPosition),
    Discrete(DiscreteValue),
}

/// A complete computed property's ordered components, with atomic combination.
///
/// Incompatible components or differing discrete components make the whole
/// property discrete. Arithmetic failures never publish a partially combined
/// value. Property-specific animation rules are selected by the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedValue {
    components: Arc<[ComputedComponent]>,
}

impl ComputedValue {
    pub fn new(components: Vec<ComputedComponent>) -> Result<Self, ComputedError> {
        if components.is_empty() {
            return Err(ComputedError::new(ComputedErrorKind::EmptyComponents));
        }
        Ok(Self {
            components: components.into(),
        })
    }
    pub fn single(component: ComputedComponent) -> Self {
        Self {
            components: Arc::from([component]),
        }
    }
    pub fn components(&self) -> &[ComputedComponent] {
        &self.components
    }
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Interpolate(progress))
    }
    pub fn add(&self, other: &Self) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Add)
    }
    pub fn accumulate(&self, other: &Self) -> Result<Self, ComputedError> {
        self.combine(other, ComputedOperation::Accumulate)
    }

    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, ComputedError> {
        if !self.compatible(other, operation) {
            return Ok(match operation {
                ComputedOperation::Interpolate(progress) if progress.value() < 0.5 => self.clone(),
                _ => other.clone(),
            });
        }
        if let ComputedOperation::Interpolate(progress) = operation {
            if progress.value() == 0.0 {
                return Ok(self.clone());
            }
            if progress.value() == 1.0 {
                return Ok(other.clone());
            }
        }
        let mut components = Vec::new();
        components
            .try_reserve_exact(self.components.len())
            .map_err(|_| ComputedError::new(ComputedErrorKind::Capacity).during(operation))?;
        for (index, (first, second)) in self
            .components
            .iter()
            .zip(other.components.iter())
            .enumerate()
        {
            components.push(
                combine_component(first, second, operation)
                    .map_err(|error| error.during(operation).at(index))?,
            );
        }
        Ok(Self {
            components: components.into(),
        })
    }

    fn compatible(&self, other: &Self, operation: ComputedOperation) -> bool {
        self.components.len() == other.components.len()
            && self
                .components
                .iter()
                .zip(other.components.iter())
                .all(|(first, second)| match (first, second) {
                    (ComputedComponent::Numeric(first), ComputedComponent::Numeric(second)) => {
                        compatible_type(first.numeric_type(), second.numeric_type()).is_some()
                    }
                    (ComputedComponent::Ratio(first), ComputedComponent::Ratio(second)) => {
                        !matches!(operation, ComputedOperation::Interpolate(_))
                            || (!first.is_degenerate() && !second.is_degenerate())
                    }
                    (ComputedComponent::Position(_), ComputedComponent::Position(_)) => true,
                    (ComputedComponent::Discrete(first), ComputedComponent::Discrete(second)) => {
                        first == second
                    }
                    _ => false,
                })
    }
}

fn combine_component(
    first: &ComputedComponent,
    second: &ComputedComponent,
    operation: ComputedOperation,
) -> Result<ComputedComponent, ComputedError> {
    match (first, second) {
        (ComputedComponent::Numeric(first), ComputedComponent::Numeric(second)) => Ok(
            ComputedComponent::Numeric(first.combine(second, operation)?),
        ),
        (ComputedComponent::Position(first), ComputedComponent::Position(second)) => Ok(
            ComputedComponent::Position(first.combine(second, operation)?),
        ),
        (ComputedComponent::Ratio(first), ComputedComponent::Ratio(second)) => {
            Ok(ComputedComponent::Ratio(match operation {
                ComputedOperation::Interpolate(progress) => first.interpolate(*second, progress)?,
                ComputedOperation::Add => first.add(*second),
                ComputedOperation::Accumulate => first.accumulate(*second),
            }))
        }
        (ComputedComponent::Discrete(_), ComputedComponent::Discrete(second)) => {
            Ok(ComputedComponent::Discrete(second.clone()))
        }
        _ => Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes).during(operation)),
    }
}

fn finite(value: f64) -> Result<f64, ComputedError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ComputedError::new(ComputedErrorKind::NonFiniteInput))
    }
}

fn scalar(first: f64, second: f64, operation: ComputedOperation) -> Result<f64, ComputedError> {
    let value = match operation {
        ComputedOperation::Interpolate(progress) => {
            checked_interpolate(first, second, progress.value())
        }
        ComputedOperation::Add | ComputedOperation::Accumulate => {
            let value = first + second;
            if value.is_finite() {
                Ok(value)
            } else {
                Err(value)
            }
        }
    };
    value.map_err(|_| ComputedError::new(ComputedErrorKind::NonFiniteResult).during(operation))
}
