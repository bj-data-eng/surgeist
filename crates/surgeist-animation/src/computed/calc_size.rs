//! Symbolic intrinsic-size calculations with bounded interpolation preparation.

use std::sync::Arc;

use crate::interpolation::InterpolationProgress;

use super::{
    ComputedError, ComputedErrorKind, ComputedNumeric, ComputedOperation, DimensionKind, MathNode,
    MathSource, NumericLiteral, NumericRepr, NumericType,
};

const MAX_CALCULATION_NODES: usize = 4096;

/// An intrinsic sizing keyword supported by the shared box-size grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizingKeyword {
    Auto,
    Stretch,
    Contain,
    MinContent,
    MaxContent,
    FitContent,
}

/// The basis of a `calc-size()` prepared for interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcSizeBasis {
    /// A definite sizing basis that introduces no `size` binding.
    Any,
    Keyword(SizingKeyword),
    /// The canonical `100%` basis, with ordinary percentage sizing behavior.
    Percentage,
}

/// The innermost basis of a `calc-size()` expression awaiting preparation.
#[derive(Debug, Clone, Copy)]
pub enum CalcSizeBasisInput<'a> {
    Canonical(CalcSizeBasis),
    Numeric(&'a CalcSizeCalculation),
}

/// The normalized expression size supplied for one opaque calculation source.
///
/// `size_references` and `percentages` count leaf occurrences in the normalized
/// expression. Sharing an expression in memory does not reduce these counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalcSizeFootprint {
    nodes: usize,
    size_references: usize,
    percentages: usize,
}

impl CalcSizeFootprint {
    /// Checks that the leaf counts fit the expression and the expression fits
    /// the native preparation budget.
    pub fn new(
        nodes: usize,
        size_references: usize,
        percentages: usize,
    ) -> Result<Self, ComputedError> {
        if nodes == 0 || size_references > nodes || percentages > nodes - size_references {
            return Err(incompatible());
        }
        if nodes > MAX_CALCULATION_NODES {
            return Err(capacity());
        }
        Ok(Self {
            nodes,
            size_references,
            percentages,
        })
    }

    const fn single(size_references: usize, percentages: usize) -> Self {
        Self {
            nodes: 1,
            size_references,
            percentages,
        }
    }

    fn substituted(self, inserted: Self) -> Result<Self, ComputedError> {
        // Each operand has at most MAX_CALCULATION_NODES nodes, including leaves.
        Self::new(
            self.nodes + self.size_references * (inserted.nodes - 1),
            self.size_references * inserted.size_references,
            self.percentages + self.size_references * inserted.percentages,
        )
    }

    fn depercentified(self) -> Result<Self, ComputedError> {
        Self::new(
            self.nodes + 2 * self.percentages,
            self.size_references + self.percentages,
            0,
        )
    }

    fn interpolated(self, other: Self) -> Result<Self, ComputedError> {
        Self::new(
            self.nodes + other.nodes + 1,
            self.size_references + other.size_references,
            self.percentages + other.percentages,
        )
    }
}

/// One node of a computed sizing calculation's immutable evaluation plan.
///
/// Operand indices precede their use. Bindings apply only inside the named
/// operand. A consumer supplies contextual source evaluation and the canonical
/// basis size and percentage context.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcSizeNode {
    /// An ordinary computed length-percentage using the current percentage context.
    Numeric(ComputedNumeric),
    /// An opaque caller-owned expression that can use the current bindings.
    Source(MathSource),
    /// The `size` keyword, evaluated using the current size binding.
    Size,
    /// Evaluates percentages in `calculation` as ratios of the current `size`.
    Depercentify { calculation: usize },
    /// Evaluates `calculation` with `size` bound to the `size` operand result.
    Substitute { calculation: usize, size: usize },
    /// Combines the two calculations using the same incoming bindings.
    Interpolate {
        first: usize,
        second: usize,
        progress: InterpolationProgress,
    },
}

/// A checked computed length-percentage calculation that can refer to `size`.
#[derive(Debug, Clone, PartialEq)]
pub struct CalcSizeCalculation {
    nodes: Arc<[CalcSizeNode]>,
    footprint: CalcSizeFootprint,
    has_opaque_numeric_terms: bool,
}

impl CalcSizeCalculation {
    /// Captures a checked numeric expression and its normalized footprint.
    ///
    /// The caller's source mapping must preserve the captured expression and
    /// evaluate it using the size and percentage bindings of the plan node.
    pub fn source(source: MathSource, footprint: CalcSizeFootprint) -> Result<Self, ComputedError> {
        require_length_percentage(source.numeric_type())?;
        Ok(Self::single(CalcSizeNode::Source(source), footprint))
    }

    /// Uses an ordinary computed length-percentage with its normal bindings.
    ///
    /// Unexpanded [`MathSource`] terms in a numeric plan have no percentage
    /// occurrence information. For a numeric basis that uses opaque sources,
    /// capture the normalized expression with [`Self::source`] instead.
    pub fn from_numeric(value: ComputedNumeric) -> Result<Self, ComputedError> {
        require_length_percentage(value.numeric_type())?;
        let (footprint, has_opaque_numeric_terms) = numeric_footprint(&value)?;
        Ok(Self {
            nodes: Arc::from([CalcSizeNode::Numeric(value)]),
            footprint,
            has_opaque_numeric_terms,
        })
    }

    /// Uses the current `size` binding as the whole calculation.
    pub fn size() -> Self {
        Self::single(CalcSizeNode::Size, CalcSizeFootprint::single(1, 0))
    }

    /// Borrows the postorder plan. The last node is the result.
    pub fn nodes(&self) -> &[CalcSizeNode] {
        &self.nodes
    }

    fn single(node: CalcSizeNode, footprint: CalcSizeFootprint) -> Self {
        Self {
            nodes: Arc::from([node]),
            footprint,
            has_opaque_numeric_terms: false,
        }
    }

    fn depercentify(&self) -> Result<Self, ComputedError> {
        let footprint = self.footprint.depercentified()?;
        self.extend(None, footprint, |calculation, _| {
            CalcSizeNode::Depercentify { calculation }
        })
    }

    fn substitute(&self, inserted: &Self) -> Result<Self, ComputedError> {
        if self.footprint.size_references == 0 {
            return Ok(self.clone());
        }
        let footprint = self.footprint.substituted(inserted.footprint)?;
        self.extend(Some(inserted), footprint, |calculation, size| {
            CalcSizeNode::Substitute { calculation, size }
        })
    }

    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ComputedError> {
        if let ([CalcSizeNode::Numeric(first)], [CalcSizeNode::Numeric(second)]) =
            (self.nodes(), other.nodes())
        {
            return Self::from_numeric(first.interpolate(second, progress)?);
        }
        let footprint = self.footprint.interpolated(other.footprint)?;
        self.extend(Some(other), footprint, |first, second| {
            CalcSizeNode::Interpolate {
                first,
                second,
                progress,
            }
        })
    }

    fn fits_interpolation(&self, other: &Self) -> bool {
        self.nodes.len() + other.nodes.len() < MAX_CALCULATION_NODES
            && self.footprint.interpolated(other.footprint).is_ok()
    }

    fn extend(
        &self,
        other: Option<&Self>,
        footprint: CalcSizeFootprint,
        operation: impl FnOnce(usize, usize) -> CalcSizeNode,
    ) -> Result<Self, ComputedError> {
        let first = self.nodes.len() - 1;
        let input_count = self.nodes.len() + other.map_or(0, |other| other.nodes.len());
        if input_count >= MAX_CALCULATION_NODES {
            return Err(capacity());
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(input_count + 1)
            .map_err(|_| capacity())?;
        nodes.extend_from_slice(&self.nodes);
        if let Some(other) = other {
            let offset = nodes.len();
            nodes.extend(other.nodes.iter().map(|node| match node {
                CalcSizeNode::Depercentify { calculation } => CalcSizeNode::Depercentify {
                    calculation: calculation + offset,
                },
                CalcSizeNode::Substitute { calculation, size } => CalcSizeNode::Substitute {
                    calculation: calculation + offset,
                    size: size + offset,
                },
                CalcSizeNode::Interpolate {
                    first,
                    second,
                    progress,
                } => CalcSizeNode::Interpolate {
                    first: first + offset,
                    second: second + offset,
                    progress: *progress,
                },
                node => node.clone(),
            }));
        }
        nodes.push(operation(first, input_count - 1));
        Ok(Self {
            nodes: nodes.into(),
            footprint,
            has_opaque_numeric_terms: self.has_opaque_numeric_terms
                || other.is_some_and(|other| other.has_opaque_numeric_terms),
        })
    }
}

/// A computed `calc-size()` with its basis prepared for interpolation.
///
/// The basis remains separate from the calculation. No intrinsic width,
/// percentage basis, or used geometry is resolved by Animation.
#[derive(Debug, Clone, PartialEq)]
pub struct CalcSize {
    basis: CalcSizeBasis,
    calculation: CalcSizeCalculation,
}

impl CalcSize {
    /// Checks a calculation against an already canonical basis.
    pub fn new(
        basis: CalcSizeBasis,
        calculation: CalcSizeCalculation,
    ) -> Result<Self, ComputedError> {
        if basis == CalcSizeBasis::Any && calculation.footprint.size_references > 0 {
            return Err(incompatible());
        }
        Ok(Self { basis, calculation })
    }

    /// Prepares the outermost calculation and its nested basis calculations.
    ///
    /// Supply each nested calculation from the outermost basis inward, then
    /// describe the innermost basis with `basis`. Substitution follows that
    /// order. Once the expression has no free `size` reference, later
    /// calculations contribute no terms, though the innermost basis still
    /// supplies its sizing behavior.
    pub fn prepare(
        basis: CalcSizeBasisInput<'_>,
        calculation: &CalcSizeCalculation,
        nested_calculations: &[CalcSizeCalculation],
    ) -> Result<Self, ComputedError> {
        let innermost = nested_calculations.last().unwrap_or(calculation);
        if matches!(basis, CalcSizeBasisInput::Canonical(CalcSizeBasis::Any))
            && innermost.footprint.size_references != 0
        {
            return Err(incompatible());
        }
        let mut expanded = calculation.clone();
        for nested in nested_calculations {
            if expanded.footprint.size_references == 0 {
                break;
            }
            expanded = expanded.substitute(nested)?;
        }
        match basis {
            CalcSizeBasisInput::Canonical(basis) => Self::new(basis, expanded),
            CalcSizeBasisInput::Numeric(basis) => Self::with_numeric_basis(basis, &expanded),
        }
    }

    /// Prepares a numeric sum basis by substitution, retaining `100%` for a
    /// sum with percentages and using `any` for a pure-length sum. Opaque
    /// native numeric bases require the normalized shape supplied by
    /// [`CalcSizeCalculation::source`].
    pub fn with_numeric_basis(
        basis: &CalcSizeCalculation,
        calculation: &CalcSizeCalculation,
    ) -> Result<Self, ComputedError> {
        if basis.footprint.size_references > 0 || basis.has_opaque_numeric_terms {
            return Err(incompatible());
        }
        if basis.footprint.percentages == 0 {
            return Self::new(CalcSizeBasis::Any, calculation.substitute(basis)?);
        }
        let prepared = if calculation.footprint.size_references == 0 {
            calculation.clone()
        } else {
            calculation.substitute(&basis.depercentify()?)?
        };
        Self::new(CalcSizeBasis::Percentage, prepared)
    }

    /// Promotes an intrinsic keyword to `calc-size(keyword, size)`.
    pub fn from_keyword(keyword: SizingKeyword) -> Self {
        Self {
            basis: CalcSizeBasis::Keyword(keyword),
            calculation: CalcSizeCalculation::size(),
        }
    }

    /// Promotes a length-percentage to `calc-size(any, value)`.
    pub fn from_numeric(value: ComputedNumeric) -> Result<Self, ComputedError> {
        Self::new(
            CalcSizeBasis::Any,
            CalcSizeCalculation::from_numeric(value)?,
        )
    }

    pub const fn basis(&self) -> CalcSizeBasis {
        self.basis
    }

    pub const fn calculation(&self) -> &CalcSizeCalculation {
        &self.calculation
    }

    /// Reports whether the canonical bases and plan budget allow interpolation.
    pub fn can_interpolate(&self, other: &Self) -> bool {
        self.interpolated_basis(other).is_some()
    }

    /// Interpolates two compatible calculations and retains their common basis.
    ///
    /// Returns `None` for different concrete bases or pairs beyond the preparation
    /// budget. Exact endpoints preserve their own bases for reuse as computed
    /// operands.
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Option<Self>, ComputedError> {
        let Some(basis) = self.interpolated_basis(other) else {
            return Ok(None);
        };
        if progress.value() == 0.0 {
            return Ok(Some(self.clone()));
        }
        if progress.value() == 1.0 {
            return Ok(Some(other.clone()));
        }
        let calculation = self
            .calculation
            .interpolate(&other.calculation, progress)
            .map_err(|error| error.during(ComputedOperation::Interpolate(progress)))?;
        Ok(Some(Self { basis, calculation }))
    }

    fn interpolated_basis(&self, other: &Self) -> Option<CalcSizeBasis> {
        if !self.calculation.fits_interpolation(&other.calculation) {
            return None;
        }
        match (self.basis, other.basis) {
            (CalcSizeBasis::Any, basis) | (basis, CalcSizeBasis::Any) => Some(basis),
            (first, second) if first == second => Some(first),
            _ => None,
        }
    }
}

fn numeric_footprint(value: &ComputedNumeric) -> Result<(CalcSizeFootprint, bool), ComputedError> {
    fn literal(value: NumericLiteral) -> (usize, usize) {
        match value {
            NumericLiteral::Mix(_) => (3, 1),
            NumericLiteral::Percentage(_) => (1, 1),
            _ => (1, 0),
        }
    }

    let plan = match &value.repr {
        NumericRepr::Literal(value) => {
            let (nodes, percentages) = literal(*value);
            return Ok((CalcSizeFootprint::new(nodes, 0, percentages)?, false));
        }
        NumericRepr::Plan(plan) => plan,
    };
    let mut nodes = 0;
    let mut percentages = 0;
    let mut has_opaque_numeric_terms = false;
    for node in plan.nodes.iter() {
        let (term_nodes, term_percentages) = match node {
            MathNode::Literal(value) => literal(*value),
            MathNode::Source(_) => {
                has_opaque_numeric_terms = true;
                (1, 0)
            }
            MathNode::Combine { .. } => (1, 0),
        };
        nodes += term_nodes;
        percentages += term_percentages;
        if nodes > MAX_CALCULATION_NODES {
            return Err(capacity());
        }
    }
    Ok((
        CalcSizeFootprint::new(nodes, 0, percentages)?,
        has_opaque_numeric_terms,
    ))
}

pub(super) fn require_length_percentage(numeric_type: NumericType) -> Result<(), ComputedError> {
    match numeric_type {
        NumericType::Dimension(DimensionKind::Length)
        | NumericType::DimensionPercentage(DimensionKind::Length)
        | NumericType::Percentage => Ok(()),
        _ => Err(incompatible()),
    }
}

fn incompatible() -> ComputedError {
    ComputedError::new(ComputedErrorKind::IncompatibleTypes)
}

fn capacity() -> ComputedError {
    ComputedError::new(ComputedErrorKind::Capacity)
}
