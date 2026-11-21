//! HTML 877d6146487f meter/progress effective numeric states.
//! Author conformance and rendering are separate from these recovered facts.

use super::*;
use surgeist_html::{
    elements::range_indicator::{HtmlMeterRange, HtmlProgressMaximum},
    microsyntax::numeric::HtmlFloatingPointNumber,
    values::Attribute,
    vocabulary::KnownHtmlElement,
};

use surgeist_html::bounded::numeric::{
    HtmlFloatingPointAllocationError, HtmlFloatingPointParseMode, try_floating_point_number,
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
            try_floating_point_number(raw.value(), HtmlFloatingPointParseMode::Recovery, remaining);
        let capacity = match &result {
            Ok(result) => result.capacity_bytes(),
            Err(HtmlFloatingPointAllocationError::ActualCapacityLimitExceeded {
                capacity_bytes,
                ..
            }) => *capacity_bytes,
            _ => 0,
        };
        self.record_allocated_bytes(capacity)?;
        let result = result.map_err(|error| {
            let kind = match error {
                HtmlFloatingPointAllocationError::RepresentationExhausted => {
                    HtmlSemanticFailureKind::RepresentationExhausted
                }
                HtmlFloatingPointAllocationError::AllocationFailed { .. } => {
                    HtmlSemanticFailureKind::AllocationFailed
                }
                HtmlFloatingPointAllocationError::RequestedCapacityLimitExceeded {
                    required,
                    ..
                } => match self.snapshot().allocated_bytes.checked_add(required) {
                    Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::AllocatedBytes,
                        limit: self.limits.allocated_bytes,
                        required,
                    },
                    None => HtmlSemanticFailureKind::RepresentationExhausted,
                },
                HtmlFloatingPointAllocationError::ActualCapacityLimitExceeded { .. } => {
                    HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::AllocatedBytes,
                        limit: self.limits.allocated_bytes,
                        required: self.snapshot().allocated_bytes,
                    }
                }
            };
            self.stop(kind)
        })?;
        self.poll()?;
        Ok(result.into_parts().0.ok())
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
        let minimum = self.range_number(min_raw)?;
        let max_raw = self.attribute(element, "max")?;
        let range = HtmlMeterRange::new(minimum, self.range_number(max_raw)?);
        let value_raw = self.attribute(element, "value")?;
        let value = range.value(self.range_number(value_raw)?);
        let low_raw = self.attribute(element, "low")?;
        let low = self.range_number(low_raw)?;
        let high_raw = self.attribute(element, "high")?;
        let high = range.high(low, self.range_number(high_raw)?);
        let optimum_raw = self.attribute(element, "optimum")?;
        let optimum = range.optimum(self.range_number(optimum_raw)?);
        Ok(HtmlSemanticValue::Known(HtmlMeterState {
            minimum: range.minimum(),
            maximum: range.maximum(),
            value,
            low: range.low(low),
            high,
            optimum,
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
        let maximum = HtmlProgressMaximum::new(self.range_number(max_raw)?);
        let value = self.range_number(value_raw)?;
        Ok(HtmlSemanticValue::Known(HtmlProgressState::Determinate(
            HtmlDeterminateProgress {
                maximum: maximum.maximum(),
                value: maximum.determinate_value(value),
                position: maximum.determinate_position(value),
            },
        )))
    }
}
