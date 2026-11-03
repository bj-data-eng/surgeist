//! The nonce getter observes the captured slot only (HTML #dom-noncedelement-nonce).
use super::*;
use surgeist_html::values::DocumentString;
impl HtmlSemanticSession<'_> {
    pub(super) fn nonce_readback(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        self.charge(surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND)?;
        if !surgeist_html::bounded::security::is_nonce_element(element.name()) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.charge(1)?;
        match element.cryptographic_nonce() {
            surgeist_html::security::nonce::HtmlCapturedNonce::Unavailable => Ok(
                HtmlSemanticValue::Missing(HtmlSemanticMissing::NonceSlot(subject.clone())),
            ),
            surgeist_html::security::nonce::HtmlCapturedNonce::Value(value) => {
                self.charge(1)?;
                Ok(HtmlSemanticValue::Known(value.clone()))
            }
        }
    }
}
