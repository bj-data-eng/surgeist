//! Specified-phase keyframe matching, independent of authored occurrence equality.

use std::fmt;

use crate::component::CssValueOrigin;
use crate::numeric::projection::{DeferredNumericProjection, prepare_keyframe_matching};
use crate::rule::keyframes::{CssKeyframeSelector, CssKeyframeSelectorList};
use crate::serialization::specified_serialization::{
    CssSpecifiedValueSerializationErrorKind, SpecifiedSerializationContext,
};
use crate::serialization::{CssSerializationError, CssSerializationLimits};

/// The work stage that rejected one complete normalized comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssKeyframeSelectorComparisonStage {
    /// Normalizing the left selector list.
    LeftNormalization,
    /// Normalizing the right selector list.
    RightNormalization,
    /// Comparing the normalized lists.
    Comparison,
}

impl CssKeyframeSelectorComparisonStage {
    const fn description(self) -> &'static str {
        match self {
            Self::LeftNormalization => "normalizing the left selectors",
            Self::RightNormalization => "normalizing the right selectors",
            Self::Comparison => "comparing the normalized selectors",
        }
    }
}

/// Atomic normalization/comparison failure with the owning native resource cause.
///
/// The optional index identifies a real selector; `None` identifies list aggregate
/// work. Origins are actual retained calculation origins. Literal/endpoints and
/// programmatic lists do not acquire fabricated positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframeSelectorComparisonError {
    cause: CssSerializationError,
    stage: CssKeyframeSelectorComparisonStage,
    selector_index: Option<usize>,
    left_origin: Option<Box<CssValueOrigin>>,
    right_origin: Option<Box<CssValueOrigin>>,
}

impl CssKeyframeSelectorComparisonError {
    /// Returns the serialization or resource failure that rejected the comparison.
    #[must_use]
    pub fn cause(&self) -> &CssSerializationError {
        &self.cause
    }
    /// Returns the stage that was running when the comparison failed.
    #[must_use]
    pub const fn stage(&self) -> CssKeyframeSelectorComparisonStage {
        self.stage
    }
    /// Returns the index of the selector being processed when the failure occurred, or `None` when the failure arose in list-level work.
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        self.selector_index
    }
    /// Returns the origin of the left-hand calculation involved in the failure, or `None` when there is none (literal endpoints and programmatic lists have no origin).
    #[must_use]
    pub fn left_origin(&self) -> Option<&CssValueOrigin> {
        self.left_origin.as_deref()
    }
    /// Returns the origin of the right-hand calculation involved in the failure, or `None` when there is none (literal endpoints and programmatic lists have no origin).
    #[must_use]
    pub fn right_origin(&self) -> Option<&CssValueOrigin> {
        self.right_origin.as_deref()
    }
}
impl fmt::Display for CssKeyframeSelectorComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "keyframe selector comparison failed while {}",
            self.stage.description()
        )?;
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

enum NormalizedOffset<'a> {
    Literal(f64),
    Calculation(DeferredNumericProjection<'a>),
}

fn origin(selector: &CssKeyframeSelector) -> Option<&CssValueOrigin> {
    match selector {
        CssKeyframeSelector::Percent(value) => value.calculation().map(|v| v.origin()),
        _ => None,
    }
}

fn normalize<'a>(
    list: &'a CssKeyframeSelectorList,
    context: &mut SpecifiedSerializationContext,
    stage: CssKeyframeSelectorComparisonStage,
) -> Result<Vec<NormalizedOffset<'a>>, CssKeyframeSelectorComparisonError> {
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
                CssSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                ),
                Some(index),
            )
        })?;
        let mut prepare = || -> Result<NormalizedOffset<'a>, CssSerializationError> {
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
    /// use surgeist_css::rule::keyframes::CssParsedKeyframeSelectors;
    /// use surgeist_css::CssParser;
    /// let left = CssParsedKeyframeSelectors::parse(&CssParser::default(), "from, calc(5% + 5%)");
    /// let right = CssParsedKeyframeSelectors::parse(&CssParser::default(), "0%, calc( 10% )");
    /// assert!(left.syntax().as_ref().unwrap().selectors()
    ///     .matches_normalized(right.syntax().as_ref().unwrap().selectors(), Default::default())
    ///     .unwrap());
    /// ```
    ///
    /// Shares cumulative input/projection limits across both lists and comparison.
    ///
    /// Each list aggregate and selector costs one input and projection node;
    /// calculation normalization uses the existing native tariffs. Each selector
    /// pair and each normalized numeric-node pair costs one projection node.
    /// No CSS is emitted, so the byte limit is unused (zero is sufficient).
    /// Failure returns no comparison result, leaves inputs unchanged, and permits
    /// retry. CSSOM retains last-match selection and live object operations.
    pub fn matches_normalized(
        &self,
        other: &Self,
        limits: CssSerializationLimits,
    ) -> Result<bool, CssKeyframeSelectorComparisonError> {
        CssKeyframeSelectorMatcher::try_new(self, limits)?.matches(other)
    }
}

/// One request-local query normalization and cumulative comparison meter.
///
/// CSSOM can iterate live candidates in reverse and choose its last match while
/// sharing all query/candidate/native work. This value is deliberately not
/// cloneable. It retains no live identity and performs no collection mutation.
pub struct CssKeyframeSelectorMatcher<'query> {
    query: &'query CssKeyframeSelectorList,
    normalized: Vec<NormalizedOffset<'query>>,
    context: SpecifiedSerializationContext,
}
impl<'query> CssKeyframeSelectorMatcher<'query> {
    /// Normalizes the query exactly once under the native cumulative budget.
    pub fn try_new(
        query: &'query CssKeyframeSelectorList,
        limits: CssSerializationLimits,
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
            let mut compare = || -> Result<bool, CssSerializationError> {
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
