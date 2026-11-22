//! Guarded background/mask layer sizes, without image or box resolution.

use super::{
    ComputedError, ComputedErrorKind, ComputedNumeric, ComputedOperation, DimensionKind,
    NumericType,
};

/// One explicit size axis: Auto or a computed length-percentage.
///
/// Numeric values retain canonical pixel lengths, percentage terms and symbolic
/// sources. Negative intermediate values remain reusable until the consumer's
/// final range handling. The caller validates authored ranges and supplies the
/// applicable percentage basis; no image geometry is read here.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerSizeAxis {
    numeric: Option<ComputedNumeric>,
}

impl LayerSizeAxis {
    /// Supplies an explicit Auto axis without resolving a natural image size.
    pub const fn auto() -> Self {
        Self { numeric: None }
    }

    /// Checks the Length, Percentage or LengthPercentage computed domain.
    pub fn numeric(value: ComputedNumeric) -> Result<Self, ComputedError> {
        match value.numeric_type() {
            NumericType::Dimension(DimensionKind::Length)
            | NumericType::DimensionPercentage(DimensionKind::Length)
            | NumericType::Percentage => Ok(Self {
                numeric: Some(value),
            }),
            _ => Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes)),
        }
    }

    /// Returns the numeric operand, or None for Auto.
    pub const fn numeric_value(&self) -> Option<&ComputedNumeric> {
        self.numeric.as_ref()
    }

    fn compatible(&self, other: &Self) -> bool {
        self.numeric.is_some() == other.numeric.is_some()
    }

    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, ComputedError> {
        match (&self.numeric, &other.numeric) {
            (None, None) => Ok(Self::auto()),
            (Some(first), Some(second)) => Self::numeric(first.combine(second, operation)?),
            _ => Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes).during(operation)),
        }
    }
}

/// A computed layer size shared by background-size and mask-size.
///
/// Explicit sizes require both axes, including any caller-lowered Auto axis.
/// Contain and Cover remain whole-size branches with no invented dimensions.
/// Matching Auto/Contain/Cover values are constant while adjacent numeric axes
/// combine. They are distinct from ordinary discrete components, whose generic
/// repeatable-list behavior still selects the complete original endpoint.
///
/// Wrap each layer in [`super::ComputedComponent::Size`], then use the existing
/// [`super::RepeatableList`] and normal interpolation/composition interfaces.
/// A mismatch anywhere in the repeated list switches the whole original list;
/// addition and accumulation instead select the original effect operand.
/// No parser, natural-size calculation, aspect-ratio resolution or clamping is
/// provided. Callers retain the correct background/mask percentage context.
///
/// ```
/// use surgeist_animation::computed::{
///     ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue,
///     DimensionKind, LayerSize, LayerSizeAxis, RepeatableList,
/// };
/// use surgeist_animation::interpolation::InterpolationProgress;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let size = |width| Ok::<_, Box<dyn std::error::Error>>(ComputedValue::single(
///     ComputedComponent::Size(LayerSize::Explicit {
///         horizontal: LayerSizeAxis::numeric(ComputedNumeric::dimension(
///             ComputedDimension::new(DimensionKind::Length, width)?,
///         ))?,
///         vertical: LayerSizeAxis::auto(),
///     }),
/// ));
/// let first = RepeatableList::new(vec![size(10.0)?])?;
/// let second = RepeatableList::new(vec![size(30.0)?])?;
/// assert_eq!(first.interpolate(&second, InterpolationProgress::new(0.5)?)?,
///            RepeatableList::new(vec![size(20.0)?])?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum LayerSize {
    Explicit {
        horizontal: LayerSizeAxis,
        vertical: LayerSizeAxis,
    },
    Contain,
    Cover,
}

impl LayerSize {
    pub(super) fn compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Contain, Self::Contain) | (Self::Cover, Self::Cover) => true,
            (
                Self::Explicit {
                    horizontal: first_x,
                    vertical: first_y,
                },
                Self::Explicit {
                    horizontal: second_x,
                    vertical: second_y,
                },
            ) => first_x.compatible(second_x) && first_y.compatible(second_y),
            _ => false,
        }
    }

    pub(super) fn combine(
        &self,
        other: &Self,
        operation: ComputedOperation,
    ) -> Result<Self, ComputedError> {
        match (self, other) {
            (Self::Contain, Self::Contain) | (Self::Cover, Self::Cover) => Ok(other.clone()),
            (
                Self::Explicit {
                    horizontal: first_x,
                    vertical: first_y,
                },
                Self::Explicit {
                    horizontal: second_x,
                    vertical: second_y,
                },
            ) => Ok(Self::Explicit {
                horizontal: first_x.combine(second_x, operation)?,
                vertical: first_y.combine(second_y, operation)?,
            }),
            _ => Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes).during(operation)),
        }
    }
}
