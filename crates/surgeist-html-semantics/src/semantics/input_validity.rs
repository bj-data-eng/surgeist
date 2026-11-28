//! Intrinsic validity from a captured current Input value.

use super::*;
use surgeist_html::{
    attributes::HtmlInputType,
    elements::input::{HtmlInputEmailMode, input_email_type_mismatch_with},
    vocabulary::KnownHtmlElement,
};

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn input_email_type_mismatch(
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
        if state != HtmlInputType::Email {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let current = match self.control_current_value(subject)? {
            Ok(current) => current,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let mode = if self.attribute(element, "multiple")?.is_some() {
            HtmlInputEmailMode::Multiple
        } else {
            HtmlInputEmailMode::Single
        };
        let mismatch =
            input_email_type_mismatch_with(current.code_units(), mode, || self.charge(1))?;
        Ok(HtmlSemanticValue::Known(mismatch))
    }
}
