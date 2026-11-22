//! Generic repeatable-list combination, distinct from property-specific lists.

use super::{
    ComputedComponent, ComputedError, ComputedErrorKind, ComputedOperation, ComputedValue,
};
use crate::interpolation::InterpolationProgress;

/// A nonempty computed list combined by the selected repeatable-list algorithm.
///
/// Property-specific list types, including shadows and transforms, own their
/// independent algorithms. Empty/`none` semantics must be lowered by that owner.
///
/// ```
/// use surgeist_animation::computed::{ComputedComponent, ComputedNumeric, ComputedValue, RepeatableList};
/// use surgeist_animation::interpolation::InterpolationProgress;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let item = |number| Ok::<_, Box<dyn std::error::Error>>(ComputedValue::single(
///     ComputedComponent::Numeric(ComputedNumeric::number(number)?)));
/// let a = RepeatableList::new(vec![item(0.0)?, item(20.0)?])?;
/// let b = RepeatableList::new(vec![item(10.0)?])?;
/// let value = a.interpolate(&b, InterpolationProgress::new(0.5)?)?;
/// assert_eq!(value.items(), &[item(5.0)?, item(15.0)?]);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RepeatableList {
    items: Vec<ComputedValue>,
}

impl RepeatableList {
    pub fn new(items: Vec<ComputedValue>) -> Result<Self, ComputedError> {
        if items.is_empty() {
            return Err(ComputedError::new(ComputedErrorKind::EmptyList));
        }
        Ok(Self { items })
    }

    pub fn items(&self) -> &[ComputedValue] {
        &self.items
    }

    /// Calculates the positive least-common-multiple before allocating a result.
    ///
    /// Zero lengths and unrepresentable expansion return distinct typed errors.
    pub fn combined_len(first: usize, second: usize) -> Result<usize, ComputedError> {
        if first == 0 || second == 0 {
            return Err(ComputedError::new(ComputedErrorKind::EmptyList));
        }
        let (mut a, mut b) = (first, second);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        (first / a)
            .checked_mul(second)
            .ok_or_else(|| ComputedError::new(ComputedErrorKind::ListExpansion))
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

    fn fallback(&self, other: &Self, operation: ComputedOperation) -> Self {
        match operation {
            ComputedOperation::Interpolate(progress) if progress.value() < 0.5 => self.clone(),
            _ => other.clone(),
        }
    }

    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, ComputedError> {
        // Unlike ordinary by-computed-value, ANY discrete component forces
        // whole-property fallback, including equal component values.
        if self.items.iter().chain(&other.items).any(|item| {
            item.components()
                .iter()
                .any(|component| matches!(component, ComputedComponent::Discrete(_)))
        }) {
            return Ok(self.fallback(other, operation));
        }
        let length = Self::combined_len(self.items.len(), other.items.len())
            .map_err(|error| error.during(operation))?;
        let mut items = Vec::new();
        items
            .try_reserve_exact(length)
            .map_err(|_| ComputedError::new(ComputedErrorKind::Capacity).during(operation))?;
        // Both original lengths are positive by construction; modulo indices
        // stay in bounds. Preflight the complete structure before arithmetic so
        // a later incompatibility selects the original whole endpoints.
        for index in 0..length {
            if !self.items[index % self.items.len()]
                .compatible(&other.items[index % other.items.len()], operation)
            {
                return Ok(self.fallback(other, operation));
            }
        }
        for index in 0..length {
            items.push(
                self.items[index % self.items.len()]
                    .combine(&other.items[index % other.items.len()], operation)
                    .map_err(|error| error.at(index))?,
            );
        }
        Ok(Self { items })
    }
}
