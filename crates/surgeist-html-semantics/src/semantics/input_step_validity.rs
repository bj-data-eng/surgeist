//! Intrinsic step membership from current Input and authored step-base evidence.

use super::*;
use surgeist_html::{
    attributes::{
        HtmlInputType,
        input_bounds::{
            HtmlInputNumericContent, HtmlInputNumericStep, select_input_numeric_constraints,
        },
    },
    bounded::{
        input::{
            HtmlInputTemporalStepError, HtmlInputTemporalStepMeter, decode_input_temporal_step,
            input_binary64_step_mismatch_with, input_temporal_step_parameters,
        },
        numeric::{HtmlIntegerUnitStepError, HtmlNumericMaterializationError},
    },
    elements::input::{HtmlInputStepCoordinate, HtmlInputStepError, HtmlInputStepOperand},
    microsyntax::numeric::{HtmlFloatingPointNumber, HtmlSignedDecimal},
    vocabulary::KnownHtmlElement,
};

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn input_step_mismatch(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let kind = self.known(element)?;
        self.charge(1)?;
        if kind != Some(KnownHtmlElement::Input) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let report = self.control_input_type_report(element)?;
        self.charge(1)?;
        let kind = report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        if !kind.supports_value_as_number() {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let current = self.control_current_value(subject)?;
        self.charge(1)?;
        if matches!(current, Ok(value) if value.is_empty()) {
            return Ok(HtmlSemanticValue::Known(false));
        }
        let step = self.attribute(element, "step")?;
        let step = self.step_attribute_value(step)?;
        if let Some(raw) = step
            && self.is_any_step(raw)?
        {
            return Ok(HtmlSemanticValue::Known(false));
        }
        let current = match current {
            Ok(value) => value,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let mismatch = if matches!(kind, HtmlInputType::Number | HtmlInputType::Range) {
            self.binary_step_mismatch(kind, element, current, step)?
        } else {
            self.temporal_step_mismatch(kind, element, current, step)?
        };
        Ok(HtmlSemanticValue::Known(mismatch))
    }

    fn step_attribute_value<'s>(
        &mut self,
        raw: Option<&'s Attribute>,
    ) -> Result<Option<&'s DocumentString>, HtmlSemanticFailure> {
        let value = raw.map(Attribute::value);
        self.interpreter
            .admit_attribute_raw(value)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        Ok(value)
    }

    fn is_any_step(&mut self, raw: &DocumentString) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        if raw.len() != HtmlInputNumericStep::ANY_KEYWORD.len() {
            return Ok(false);
        }
        for (index, unit) in raw.code_units().iter().enumerate() {
            self.charge(1)?;
            if !HtmlInputNumericStep::any_keyword_unit_matches(index, *unit) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn binary_step_mismatch(
        &mut self,
        kind: HtmlInputType,
        element: &'a HtmlCapturedElement,
        current: &DocumentString,
        step: Option<&DocumentString>,
    ) -> Result<bool, HtmlSemanticFailure> {
        let Some(current) = self.recover_range_number(current)? else {
            return Ok(false);
        };
        let step = match step {
            Some(raw) => self
                .recover_range_number(raw)?
                .map_or(HtmlInputNumericStep::Failed, HtmlInputNumericStep::Value),
            None => HtmlInputNumericStep::Absent,
        };
        let minimum = self.attribute(element, "min")?;
        let minimum = self.range_number(minimum)?;
        let authored_value = if minimum.is_none() {
            let attr = self.attribute(element, "value")?;
            self.range_number(attr)?
        } else {
            None
        };
        self.charge(1)?;
        let constraints = select_input_numeric_constraints(
            minimum.map_or(
                HtmlInputNumericContent::Absent,
                HtmlInputNumericContent::Value,
            ),
            HtmlInputNumericContent::Absent,
            step,
            authored_value.map_or(
                HtmlInputNumericContent::Absent,
                HtmlInputNumericContent::Value,
            ),
            kind,
        )
        .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        let step = constraints
            .allowed_step()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        input_binary64_step_mismatch_with(current, constraints.step_base(), step, || self.charge(1))
            .map_err(|error| self.step_arithmetic_stop(error))
    }

    fn temporal_step_mismatch(
        &mut self,
        kind: HtmlInputType,
        element: &'a HtmlCapturedElement,
        current: &DocumentString,
        step: Option<&DocumentString>,
    ) -> Result<bool, HtmlSemanticFailure> {
        let Some(current) = self.step_temporal_value(kind, Some(current))? else {
            return Ok(false);
        };
        let parameters = input_temporal_step_parameters(kind)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        let minimum = self.attribute(element, "min")?;
        let minimum = self.step_attribute_value(minimum)?;
        let mut base = self.step_temporal_value(kind, minimum)?;
        if base.is_none() {
            let value = self.attribute(element, "value")?;
            let value = self.step_attribute_value(value)?;
            base = self.step_temporal_value(kind, value)?;
        }
        let step = match step {
            Some(step) => self
                .recover_range_number(step)?
                .filter(|step| step.value() > 0.0),
            None => None,
        };
        self.charge(1)?;
        let step = match step {
            Some(step) => step,
            None => self.step_finite(f64::from(parameters.default_step))?,
        };
        let mut grid = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let result = (|| {
            let [fallback, coefficient, scaled, remainder] = &mut grid;
            let base = match &base {
                Some(value) => HtmlInputStepOperand::decimal(value),
                None => {
                    let origin = self.step_finite(parameters.default_base as f64)?;
                    self.step_expanded(origin, fallback)?.operand()
                }
            };
            let step = self.step_expanded(step, coefficient)?.operand();
            self.step_digits(scaled, step.multiplication_cells())?;
            let scaled = step
                .multiply(parameters.scale, scaled, || self.charge(1))
                .map_err(|error| self.step_arithmetic_stop(error))?;
            let step = scaled.operand();
            let probe = HtmlInputStepOperand::decimal(&current);
            self.step_digits(remainder, probe.membership_cells(base, step))?;
            probe
                .step_mismatch_with(base, step, remainder, || self.charge(1))
                .map_err(|error| self.step_arithmetic_stop(error))
        })();
        for scratch in &grid {
            self.release_scratch(scratch);
        }
        result
    }

    fn step_finite(&mut self, value: f64) -> Result<HtmlFloatingPointNumber, HtmlSemanticFailure> {
        HtmlFloatingPointNumber::try_from(value)
            .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }

    fn step_expanded<'s>(
        &mut self,
        number: HtmlFloatingPointNumber,
        scratch: &'s mut Vec<u8>,
    ) -> Result<HtmlInputStepCoordinate<'s>, HtmlSemanticFailure> {
        self.step_digits(
            scratch,
            Some(HtmlInputStepCoordinate::binary64_cells(number)),
        )?;
        HtmlInputStepCoordinate::from_binary64_with(number, scratch, || self.charge(1))
            .map_err(|error| self.step_arithmetic_stop(error))
    }

    fn step_digits(
        &mut self,
        scratch: &mut Vec<u8>,
        cells: Option<usize>,
    ) -> Result<(), HtmlSemanticFailure> {
        let cells =
            cells.ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.reserve_scratch(scratch, cells)?;
        self.charge(cells)?;
        scratch.resize(cells, 0);
        Ok(())
    }

    fn step_arithmetic_stop(
        &mut self,
        error: HtmlInputStepError<HtmlSemanticFailure>,
    ) -> HtmlSemanticFailure {
        match error {
            HtmlInputStepError::Work(stop) => stop,
            HtmlInputStepError::Workspace => {
                self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
            }
            HtmlInputStepError::NonPositiveStep => {
                self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
            }
        }
    }

    fn step_temporal_value(
        &mut self,
        kind: HtmlInputType,
        raw: Option<&DocumentString>,
    ) -> Result<Option<HtmlSignedDecimal>, HtmlSemanticFailure> {
        let Some(raw) = raw else {
            return Ok(None);
        };
        let mut meter = TemporalStepMeter(self);
        let (result, _, _) =
            decode_input_temporal_step(kind, Some(raw.code_units()), usize::MAX, &mut meter)
                .into_parts();
        result.map_err(|error| {
            let kind = match error {
                HtmlInputTemporalStepError::Admission(stop) => return stop,
                HtmlInputTemporalStepError::Inapplicable => {
                    HtmlSemanticFailureKind::IntrinsicUnavailable
                }
                HtmlInputTemporalStepError::Materialization(
                    HtmlNumericMaterializationError::Boundary(
                        HtmlIntegerUnitStepError::AllocationFailed { .. },
                    ),
                ) => HtmlSemanticFailureKind::AllocationFailed,
                _ => HtmlSemanticFailureKind::RepresentationExhausted,
            };
            self.stop(kind)
        })
    }
}

struct TemporalStepMeter<'s, 'a>(&'s mut HtmlSemanticSession<'a>);

impl HtmlInputTemporalStepMeter for TemporalStepMeter<'_, '_> {
    type Stop = HtmlSemanticFailure;

    fn work(&mut self, units: usize) -> Result<(), Self::Stop> {
        self.0.charge(units)
    }

    fn reserve(
        &mut self,
        bytes: usize,
        _: usize,
        scratch_records: usize,
    ) -> Result<(), Self::Stop> {
        self.0.poll()?;
        self.0
            .account
            .check(&[
                Change::Acquire(
                    HtmlResource::ScratchRecords,
                    query_account::wide(scratch_records),
                ),
                Change::Acquire(HtmlResource::AllocatedBytes, query_account::wide(bytes)),
            ])
            .map_err(|error| self.0.ledger_stop(error))
    }

    fn observe_allocation(
        &mut self,
        bytes: usize,
        scratch_records: usize,
    ) -> Result<(), Self::Stop> {
        self.0
            .account
            .observe(&[
                Change::Acquire(
                    HtmlResource::ScratchRecords,
                    query_account::wide(scratch_records),
                ),
                Change::Acquire(HtmlResource::AllocatedBytes, query_account::wide(bytes)),
            ])
            .map_err(|error| self.0.ledger_stop(error))
    }

    fn release(&mut self, _: usize, _: usize, scratch_records: usize) {
        // AllocatedBytes remains cumulative after transient backing is dropped.
        if scratch_records != 0
            && let Err(error) = self.0.account.release_scratch(scratch_records)
        {
            self.0.ledger_stop(error);
        }
    }
}
