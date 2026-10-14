//! Captured figure caption membership, not content-model conformance or exposure.
//! HTML 877d6146487f §4.4.12:
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-figure-element>.

use super::*;

impl HtmlSemanticSession<'_> {
    pub(super) fn figure_caption(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Figure) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        Ok(
            match self.first_html_element_child(subject, KnownHtmlElement::Figcaption)? {
                Ok(caption) => HtmlSemanticValue::Known(caption),
                Err(missing) => HtmlSemanticValue::Missing(missing),
            },
        )
    }
}
