//! The explicit address forbidden-descendant restriction, HTML 877d6146487f:
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-address-element>.
//! Heading and sectioning categories follow the same selected edition:
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#heading-content-2>
//! and <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sectioning-content-2>.
//! This predicate does not certify flow content or contact-information meaning.

use super::*;

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn address_has_forbidden_descendant(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::Address) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let node = self.node(subject)?;
        let mut frames = Vec::new();
        let result = self.address_descendants(node, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }

    fn address_descendants(
        &mut self,
        root: HtmlCapturedNode<'a>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = match self.ordinary_tree_next(root, frames)? {
            Ok(current) => current,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let mut deferred_missing = None;
        while let Some(id) = current {
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && matches!(
                    self.known(element)?,
                    Some(
                        KnownHtmlElement::H1
                            | KnownHtmlElement::H2
                            | KnownHtmlElement::H3
                            | KnownHtmlElement::H4
                            | KnownHtmlElement::H5
                            | KnownHtmlElement::H6
                            | KnownHtmlElement::Article
                            | KnownHtmlElement::Aside
                            | KnownHtmlElement::Nav
                            | KnownHtmlElement::Section
                            | KnownHtmlElement::Header
                            | KnownHtmlElement::Footer
                            | KnownHtmlElement::Address
                    )
                )
            {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(true));
            }
            // A heading-bearing hgroup also contains a prohibited h1–h6.
            // Traversing its ordinary children proves that case without treating
            // a heading-free hgroup as forbidden or adding a second category walk.
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(current) => current,
                Err(missing) => {
                    if deferred_missing.is_none() {
                        self.charge(1)?;
                        deferred_missing = Some(missing);
                    }
                    self.ordinary_tree_resume(frames)?
                }
            };
        }
        self.charge(1)?;
        Ok(match deferred_missing {
            Some(missing) => HtmlSemanticValue::Missing(missing),
            None => HtmlSemanticValue::Known(false),
        })
    }
}
