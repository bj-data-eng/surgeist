//! Intrinsic Input range facts from the captured current value and authored bounds.

use super::*;
use surgeist_html::{
    attributes::{
        HtmlInputType,
        input_bounds::{
            HtmlInputNumericContent, HtmlInputNumericStep, select_input_numeric_constraints,
        },
    },
    bounded::input::{HtmlInputValueConversionError, input_temporal_range_validity_with},
    elements::input::HtmlInputRangeValidity,
    vocabulary::KnownHtmlElement,
};

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn input_range_validity(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlInputRangeValidity>, HtmlSemanticFailure> {
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
            return Ok(HtmlSemanticValue::Known(HtmlInputRangeValidity::default()));
        }

        let minimum = self.attribute(element, "min")?;
        let maximum = self.attribute(element, "max")?;
        if matches!(kind, HtmlInputType::Number | HtmlInputType::Range) {
            return self.binary_range_validity(kind, minimum, maximum, current);
        }

        for raw in [minimum, maximum] {
            self.interpreter
                .admit_attribute_raw(raw.map(Attribute::value))
                .map_err(|err| self.stop(HtmlSemanticFailureKind::Attribute(err)))?;
        }
        let result = input_temporal_range_validity_with(
            kind,
            minimum.map(|raw| raw.value().code_units()),
            maximum.map(|raw| raw.value().code_units()),
            current.as_ref().ok().map(|value| value.code_units()),
            |work| self.charge(work),
        )
        .map_err(|err| match err {
            HtmlInputValueConversionError::Admission(stop) => stop,
            HtmlInputValueConversionError::WorkOverflow => {
                self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
            }
            _ => self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable),
        })?;
        self.poll()?;
        match (result, current) {
            (Some(validity), _) => Ok(HtmlSemanticValue::Known(validity)),
            (None, Err(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            (None, Ok(_)) => Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
        }
    }

    fn binary_range_validity(
        &mut self,
        kind: HtmlInputType,
        minimum: Option<&Attribute>,
        maximum: Option<&Attribute>,
        current: Result<&DocumentString, HtmlSemanticMissing>,
    ) -> Result<HtmlSemanticValue<HtmlInputRangeValidity>, HtmlSemanticFailure> {
        let minimum = self.range_number(minimum)?;
        let maximum = self.range_number(maximum)?;
        self.charge(1)?;
        let limits = select_input_numeric_constraints(
            minimum.map_or(
                HtmlInputNumericContent::Absent,
                HtmlInputNumericContent::Value,
            ),
            maximum.map_or(
                HtmlInputNumericContent::Absent,
                HtmlInputNumericContent::Value,
            ),
            HtmlInputNumericStep::Absent,
            HtmlInputNumericContent::Absent,
            kind,
        )
        .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        if limits.minimum().is_none() && limits.maximum().is_none() {
            return Ok(HtmlSemanticValue::Known(HtmlInputRangeValidity::default()));
        }
        let value = match current {
            Ok(current) => self.recover_range_number(current)?,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(
            value.map_or_else(HtmlInputRangeValidity::default, |value| {
                limits.range_validity(value)
            }),
        ))
    }
}
