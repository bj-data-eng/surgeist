//! The nonce getter observes the captured slot only (HTML #dom-noncedelement-nonce).
use super::*;
impl HtmlSemanticSession<'_> {
    pub(super) fn nonce_readback(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        self.charge(crate::bounded::HTML_NONCE_INSPECTION_BOUND)?;
        if !crate::bounded::is_nonce_element(element.name()) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.charge(1)?;
        match element.cryptographic_nonce() {
            crate::HtmlCapturedNonce::Unavailable => Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::NonceSlot(subject.clone()),
            )),
            crate::HtmlCapturedNonce::Value(value) => {
                self.charge(1)?;
                Ok(HtmlSemanticValue::Known(value.clone()))
            }
        }
    }
}
