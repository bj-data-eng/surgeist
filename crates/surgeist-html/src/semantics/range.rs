//! HTML 877d6146487f meter/progress effective numeric states.
//! Author conformance and rendering are separate from these recovered facts.

use super::*;
use crate::{
    HtmlFloatingPointNumber,
    numeric::{FloatingPointAllocationError, FloatingPointParseMode, try_floating_point_number},
};

/// The source-ordered, clamped six-point meter state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlMeterState {
    minimum: HtmlFloatingPointNumber,
    maximum: HtmlFloatingPointNumber,
    value: HtmlFloatingPointNumber,
    low: HtmlFloatingPointNumber,
    high: HtmlFloatingPointNumber,
    optimum: HtmlFloatingPointNumber,
}
impl HtmlMeterState {
    pub fn minimum(&self) -> HtmlFloatingPointNumber {
        self.minimum
    }
    pub fn maximum(&self) -> HtmlFloatingPointNumber {
        self.maximum
    }
    pub fn value(&self) -> HtmlFloatingPointNumber {
        self.value
    }
    pub fn low(&self) -> HtmlFloatingPointNumber {
        self.low
    }
    pub fn high(&self) -> HtmlFloatingPointNumber {
        self.high
    }
    pub fn optimum(&self) -> HtmlFloatingPointNumber {
        self.optimum
    }
}

/// A determinate progress state; all three values are finite and bounded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlDeterminateProgress {
    maximum: HtmlFloatingPointNumber,
    value: HtmlFloatingPointNumber,
    position: HtmlFloatingPointNumber,
}
impl HtmlDeterminateProgress {
    pub fn maximum(&self) -> HtmlFloatingPointNumber {
        self.maximum
    }
    pub fn value(&self) -> HtmlFloatingPointNumber {
        self.value
    }
    pub fn position(&self) -> HtmlFloatingPointNumber {
        self.position
    }
}

/// Absence of the value attribute is distinct from present invalid text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlProgressState {
    Indeterminate,
    Determinate(HtmlDeterminateProgress),
}

impl<'a> HtmlSemanticSession<'a> {
    fn range_number(
        &mut self,
        raw: Option<&Attribute>,
    ) -> Result<Option<HtmlFloatingPointNumber>, HtmlSemanticFailure> {
        let Some(raw) = raw else {
            return Ok(None);
        };
        self.interpreter
            .admit_attribute_raw(Some(raw.value()))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let remaining = self.numeric_budget(raw.value().len())?;
        let result =
            try_floating_point_number(raw.value(), FloatingPointParseMode::Recovery, remaining);
        let capacity = match &result {
            Ok(result) => result.capacity_bytes,
            Err(FloatingPointAllocationError::ActualCapacityLimitExceeded {
                capacity_bytes,
                ..
            }) => *capacity_bytes,
            _ => 0,
        };
        self.record_allocated_bytes(capacity)?;
        let result = result.map_err(|error| {
            let kind = match error {
                FloatingPointAllocationError::RepresentationExhausted => {
                    HtmlSemanticFailureKind::RepresentationExhausted
                }
                FloatingPointAllocationError::AllocationFailed { .. } => {
                    HtmlSemanticFailureKind::AllocationFailed
                }
                FloatingPointAllocationError::RequestedCapacityLimitExceeded {
                    required, ..
                } => match self.usage.allocated_bytes.checked_add(required) {
                    Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::AllocatedBytes,
                        limit: self.limits.allocated_bytes,
                        required,
                    },
                    None => HtmlSemanticFailureKind::RepresentationExhausted,
                },
                FloatingPointAllocationError::ActualCapacityLimitExceeded { .. } => {
                    HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::AllocatedBytes,
                        limit: self.limits.allocated_bytes,
                        required: self.usage.allocated_bytes,
                    }
                }
            };
            self.stop(kind)
        })?;
        self.poll()?;
        Ok(result.value.ok())
    }

    fn range_finite(&mut self, value: f64) -> Result<HtmlFloatingPointNumber, HtmlSemanticFailure> {
        HtmlFloatingPointNumber::try_from(value)
            .map_err(|_| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))
    }

    pub(super) fn meter_state(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlMeterState>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::Meter) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.charge(20)?;
        let min_raw = self.attribute(element, "min")?;
        let minimum = self.range_number(min_raw)?.map_or(0.0, |n| n.value());
        let max_raw = self.attribute(element, "max")?;
        let maximum = self
            .range_number(max_raw)?
            .map_or(1.0, |n| n.value())
            .max(minimum);
        let value_raw = self.attribute(element, "value")?;
        let value = self
            .range_number(value_raw)?
            .map_or(0.0, |n| n.value())
            .clamp(minimum, maximum);
        let low_raw = self.attribute(element, "low")?;
        let low = self
            .range_number(low_raw)?
            .map_or(minimum, |n| n.value())
            .clamp(minimum, maximum);
        let high_raw = self.attribute(element, "high")?;
        let high = self
            .range_number(high_raw)?
            .map_or(maximum, |n| n.value())
            .clamp(low, maximum);
        let optimum_raw = self.attribute(element, "optimum")?;
        let optimum = match self.range_number(optimum_raw)? {
            Some(number) => number.value(),
            // The finite result uses Rust's midpoint operation, avoiding both
            // endpoint overflow and an early rounded/underflowed difference.
            None => minimum.midpoint(maximum),
        }
        .clamp(minimum, maximum);
        Ok(HtmlSemanticValue::Known(HtmlMeterState {
            minimum: self.range_finite(minimum)?,
            maximum: self.range_finite(maximum)?,
            value: self.range_finite(value)?,
            low: self.range_finite(low)?,
            high: self.range_finite(high)?,
            optimum: self.range_finite(optimum)?,
        }))
    }

    pub(super) fn progress_state(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlProgressState>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::Progress) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let value_raw = self.attribute(element, "value")?;
        if value_raw.is_none() {
            return Ok(HtmlSemanticValue::Known(HtmlProgressState::Indeterminate));
        }
        self.charge(8)?;
        let max_raw = self.attribute(element, "max")?;
        let maximum = self
            .range_number(max_raw)?
            .filter(|n| n.value() > 0.0)
            .map_or(1.0, |n| n.value());
        let value = self
            .range_number(value_raw)?
            .map_or(0.0, |n| n.value())
            .clamp(0.0, maximum);
        Ok(HtmlSemanticValue::Known(HtmlProgressState::Determinate(
            HtmlDeterminateProgress {
                maximum: self.range_finite(maximum)?,
                value: self.range_finite(value)?,
                position: self.range_finite(value / maximum)?,
            },
        )))
    }
}
