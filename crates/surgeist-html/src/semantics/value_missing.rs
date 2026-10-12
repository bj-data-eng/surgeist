//! HTML 877d6146487f §4.10.5.3.4 and §4.10.11: required text missingness.
//! Observes supplied current values; performs no validation or control mutation.

use super::*;
use crate::HtmlInputType;

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn input_text_value_missing(
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
        let state = report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        if !matches!(
            state,
            HtmlInputType::Text
                | HtmlInputType::Search
                | HtmlInputType::Telephone
                | HtmlInputType::Url
                | HtmlInputType::Email
                | HtmlInputType::Password
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.required_text_value_missing(subject, element)
    }

    pub(super) fn textarea_value_missing(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let kind = self.known(element)?;
        self.charge(1)?;
        if kind != Some(KnownHtmlElement::Textarea) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.required_text_value_missing(subject, element)
    }

    fn required_text_value_missing(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let required = self.attribute(element, "required")?;
        self.charge(1)?;
        if required.is_none() {
            return Ok(HtmlSemanticValue::Known(false));
        }
        // Every qualified input state above has applicable readonly; textarea
        // has the same presence-based immutability condition in its own clause.
        let readonly = self.attribute(element, "readonly")?;
        self.charge(1)?;
        if readonly.is_some() {
            return Ok(HtmlSemanticValue::Known(false));
        }
        let current_missing = match self.control_current_value(subject)? {
            Ok(value) => {
                self.charge(1)?;
                if !value.is_empty() {
                    return Ok(HtmlSemanticValue::Known(false));
                }
                None
            }
            Err(missing) => Some(missing),
        };
        let disabled = self.form_disabled(subject, false)?;
        self.charge(1)?;
        match disabled {
            HtmlSemanticValue::Known(true) => Ok(HtmlSemanticValue::Known(false)),
            HtmlSemanticValue::Known(false) => {
                Ok(current_missing
                    .map_or(HtmlSemanticValue::Known(true), HtmlSemanticValue::Missing))
            }
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(
                current_missing.unwrap_or(missing),
            )),
            HtmlSemanticValue::Inapplicable | HtmlSemanticValue::Unsupported(_) => {
                Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
            }
        }
    }
}
