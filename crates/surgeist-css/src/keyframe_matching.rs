//! Specified-phase keyframe matching, independent of authored occurrence equality.

use std::fmt;

use crate::{
    CssKeyframeSelector, CssKeyframeSelectorList, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    numeric::{DeferredNumericProjection, prepare_keyframe_matching},
    specified_serialization::SpecifiedSerializationContext,
};

/// The work stage that rejected one complete normalized comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssKeyframeSelectorComparisonStage {
    LeftNormalization,
    RightNormalization,
    Comparison,
}

/// Atomic normalization/comparison failure with the owning native resource cause.
///
/// The optional index identifies a real selector; `None` identifies list aggregate
/// work. Origins are actual retained calculation origins. Literal/endpoints and
/// programmatic lists do not acquire fabricated positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframeSelectorComparisonError {
    cause: CssSpecifiedValueSerializationError,
    stage: CssKeyframeSelectorComparisonStage,
    selector_index: Option<usize>,
    left_origin: Option<Box<CssValueOrigin>>,
    right_origin: Option<Box<CssValueOrigin>>,
}

impl CssKeyframeSelectorComparisonError {
    #[must_use]
    pub fn cause(&self) -> &CssSpecifiedValueSerializationError {
        &self.cause
    }
    #[must_use]
    pub const fn stage(&self) -> CssKeyframeSelectorComparisonStage {
        self.stage
    }
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        self.selector_index
    }
    #[must_use]
    pub fn left_origin(&self) -> Option<&CssValueOrigin> {
        self.left_origin.as_deref()
    }
    #[must_use]
    pub fn right_origin(&self) -> Option<&CssValueOrigin> {
        self.right_origin.as_deref()
    }
}
impl fmt::Display for CssKeyframeSelectorComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "keyframe selector comparison {:?}", self.stage)?;
        if let Some(index) = self.selector_index {
            write!(formatter, " at selector {index}")?;
        }
        write!(formatter, ": {}", self.cause)
    }
}
impl std::error::Error for CssKeyframeSelectorComparisonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

enum NormalizedOffset {
    Literal(f64),
    Calculation(DeferredNumericProjection),
}

fn origin(selector: &CssKeyframeSelector) -> Option<&CssValueOrigin> {
    match selector {
        CssKeyframeSelector::Percent(value) => value.calculation().map(|v| v.origin()),
        _ => None,
    }
}

fn normalize(
    list: &CssKeyframeSelectorList,
    context: &mut SpecifiedSerializationContext,
    stage: CssKeyframeSelectorComparisonStage,
) -> Result<Vec<NormalizedOffset>, CssKeyframeSelectorComparisonError> {
    let failure = |cause, index: Option<usize>| {
        let source = index
            .and_then(|index| origin(&list.selectors()[index]))
            .cloned()
            .map(Box::new);
        let (left_origin, right_origin) = match stage {
            CssKeyframeSelectorComparisonStage::LeftNormalization => (source, None),
            CssKeyframeSelectorComparisonStage::RightNormalization => (None, source),
            CssKeyframeSelectorComparisonStage::Comparison => unreachable!("normalization stage"),
        };
        CssKeyframeSelectorComparisonError {
            cause,
            stage,
            selector_index: index,
            left_origin,
            right_origin,
        }
    };
    context.charge_input(1).map_err(|e| failure(e, None))?;
    context
        .charge_generated_projection(1)
        .map_err(|e| failure(e, None))?;
    let mut offsets = Vec::new();
    for (index, selector) in list.selectors().iter().enumerate() {
        context
            .charge_input(1)
            .map_err(|e| failure(e, Some(index)))?;
        context
            .charge_generated_projection(1)
            .map_err(|e| failure(e, Some(index)))?;
        offsets.try_reserve(1).map_err(|_| {
            failure(
                CssSpecifiedValueSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                ),
                Some(index),
            )
        })?;
        let mut prepare = || -> Result<NormalizedOffset, CssSpecifiedValueSerializationError> {
            match selector {
                CssKeyframeSelector::From => Ok(NormalizedOffset::Literal(0.0)),
                CssKeyframeSelector::To => Ok(NormalizedOffset::Literal(100.0)),
                CssKeyframeSelector::Percent(percent) => {
                    if let Some(value) = percent.literal_value() {
                        Ok(NormalizedOffset::Literal(value))
                    } else {
                        let calculation = percent.calculation().expect("checked percentage phase");
                        prepare_keyframe_matching(&calculation.expression, context)
                            .map(NormalizedOffset::Calculation)
                    }
                }
            }
        };
        offsets.push(prepare().map_err(|e| failure(e, Some(index)))?);
    }
    Ok(offsets)
}

impl CssKeyframeSelectorList {
    /// Compares complete ordered sequences in the normalized specified-math phase.
    ///
    /// `from`/`to` become literal 0%/100%. Count, order and duplicates remain.
    /// Calculations simplify with the shared binary64 owner without external
    /// conversion data, then compare normalized roots without provenance or text
    /// rounding. Literal and calculation alternatives remain distinct. No range
    /// clamping or computed offset evaluation occurs; NaN compares unequal.
    /// Both complete sequences normalize before equality is decided.
    ///
    /// ```
    /// use surgeist_css::parse_keyframe_selector_list;
    /// let left = parse_keyframe_selector_list("from, calc(5% + 5%)");
    /// let right = parse_keyframe_selector_list("0%, calc( 10% )");
    /// assert!(left.syntax().as_ref().unwrap().selectors()
    ///     .matches_normalized(right.syntax().as_ref().unwrap().selectors()).unwrap());
    /// ```
    pub fn matches_normalized(
        &self,
        other: &Self,
    ) -> Result<bool, CssKeyframeSelectorComparisonError> {
        self.matches_normalized_with_limits(other, CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares cumulative input/projection limits across both lists and comparison.
    ///
    /// Each list aggregate and selector costs one input and projection node;
    /// calculation normalization uses the existing native tariffs. Each selector
    /// pair and each normalized numeric-node pair costs one projection node.
    /// No CSS is emitted, so the byte limit is unused (zero is sufficient).
    /// Failure returns no comparison result, leaves inputs unchanged, and permits
    /// retry. CSSOM retains last-match selection and live object operations.
    pub fn matches_normalized_with_limits(
        &self,
        other: &Self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<bool, CssKeyframeSelectorComparisonError> {
        CssKeyframeSelectorMatcher::try_new_with_limits(self, limits)?.matches(other)
    }
}

/// One request-local query normalization and cumulative comparison meter.
///
/// CSSOM can iterate live candidates in reverse and choose its last match while
/// sharing all query/candidate/native work. This value is deliberately not
/// cloneable. It retains no live identity and performs no collection mutation.
pub struct CssKeyframeSelectorMatcher<'query> {
    query: &'query CssKeyframeSelectorList,
    normalized: Vec<NormalizedOffset>,
    context: SpecifiedSerializationContext,
}
impl<'query> CssKeyframeSelectorMatcher<'query> {
    pub fn try_new(
        query: &'query CssKeyframeSelectorList,
    ) -> Result<Self, CssKeyframeSelectorComparisonError> {
        Self::try_new_with_limits(query, CssSpecifiedValueSerializationLimits::default())
    }

    /// Normalizes the query exactly once under the native cumulative budget.
    pub fn try_new_with_limits(
        query: &'query CssKeyframeSelectorList,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssKeyframeSelectorComparisonError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let normalized = normalize(
            query,
            &mut context,
            CssKeyframeSelectorComparisonStage::LeftNormalization,
        )?;
        Ok(Self {
            query,
            normalized,
            context,
        })
    }

    /// Normalizes and compares one complete candidate using the same meter.
    ///
    /// Completed work is never refunded, including work before a failure. No
    /// result is returned on failure. Retry the unchanged query/candidates with
    /// a fresh matcher and larger limits; authored inputs remain unchanged.
    pub fn matches(
        &mut self,
        other: &CssKeyframeSelectorList,
    ) -> Result<bool, CssKeyframeSelectorComparisonError> {
        use CssKeyframeSelectorComparisonStage as Stage;
        let right = normalize(other, &mut self.context, Stage::RightNormalization)?;
        if self.normalized.len() != right.len() {
            return Ok(false);
        }
        let mut equal = true;
        for (index, (a, b)) in self.normalized.iter().zip(&right).enumerate() {
            let mut compare = || -> Result<bool, CssSpecifiedValueSerializationError> {
                self.context.charge_generated_projection(1)?;
                match (a, b) {
                    (NormalizedOffset::Literal(a), NormalizedOffset::Literal(b)) => Ok(a == b),
                    (NormalizedOffset::Calculation(a), NormalizedOffset::Calculation(b)) => {
                        a.same_specified_root(b, &mut self.context)
                    }
                    _ => Ok(false),
                }
            };
            equal &= compare().map_err(|cause| CssKeyframeSelectorComparisonError {
                cause,
                stage: Stage::Comparison,
                selector_index: Some(index),
                left_origin: origin(&self.query.selectors()[index])
                    .cloned()
                    .map(Box::new),
                right_origin: origin(&other.selectors()[index]).cloned().map(Box::new),
            })?;
        }
        Ok(equal)
    }
}
