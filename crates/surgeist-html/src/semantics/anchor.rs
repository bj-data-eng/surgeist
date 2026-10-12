//! Exact captured HTMLAnchorElement.text getter, HTML 877d6146487f:
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-a-text>.
//! Descendant text content follows the retained DOM definition:
//! <https://dom.spec.whatwg.org/#concept-descendant-text-content>.
//! This does not implement the setter, rendered text or hyperlink activation.

use super::*;

impl HtmlSemanticSession<'_> {
    pub(super) fn anchor_text(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::A) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.control_text_content(subject, HtmlControlTextOrigin::DescendantText)
    }
}
