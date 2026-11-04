//! Captured footer scope, distinct from author content conformance and exposure.
//! HTML 877d6146487f §4.3.9, §3.2.5.2.3 and §3.1.7:
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-footer-element>,
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sectioning-content>,
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-body-element-2>.

use super::*;
use crate::HtmlCaptureDocumentId;
use surgeist_html::vocabulary::KnownHtmlElement;

/// Source-selected scope of a captured HTML footer.
/// A null document body is established absence, not unavailable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlFooterScope {
    Section(HtmlCaptureSubjectId),
    DocumentBody {
        document: HtmlCaptureDocumentId,
        body: Option<HtmlCaptureSubjectId>,
    },
}

/// What ended a document body search without an answer.
enum BodyStop {
    Missing(HtmlSemanticMissing),
    Unsupported(HtmlSemanticUnsupported),
}
impl HtmlSemanticSession<'_> {
    pub(super) fn footer_scope(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlFooterScope>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Footer) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut current = node;
        loop {
            let parent = match self.parent(current)? {
                Ok(Some(parent)) => parent,
                Ok(None) => break,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            current = self.node(&parent)?;
            if let HtmlCapturedNodeKind::Element(element) = current.kind()
                && matches!(
                    self.known(element)?,
                    Some(
                        KnownHtmlElement::Article
                            | KnownHtmlElement::Aside
                            | KnownHtmlElement::Nav
                            | KnownHtmlElement::Section
                    )
                )
            {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(HtmlFooterScope::Section(parent)));
            }
        }
        self.charge(1)?;
        let document = node.node_document();
        let body = match self.footer_document_body(document.subject())? {
            Ok(body) => body,
            Err(BodyStop::Missing(missing)) => return Ok(HtmlSemanticValue::Missing(missing)),
            Err(BodyStop::Unsupported(reason)) => {
                return Ok(HtmlSemanticValue::Unsupported(reason));
            }
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlFooterScope::DocumentBody {
            document,
            body,
        }))
    }

    // The document element must be unique before classifying its HTML name or
    // reading its children. Permissive capture multiplicity is query-local
    // unsupported evidence, not a first-of-many body selection.
    fn footer_document_body(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, BodyStop>, HtmlSemanticFailure> {
        let node = self.node(document)?;
        self.charge(1)?;
        let mut children = match node.children() {
            HtmlCapturedChildren::Complete(children) => children,
            HtmlCapturedChildren::Unavailable => {
                return Ok(Err(BodyStop::Missing(
                    HtmlSemanticMissing::OrderedChildren(document.clone()),
                )));
            }
        };
        let mut html = None;
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else { break };
            let node = self.node(&child)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                self.charge(1)?;
                if html.is_some() {
                    return Ok(Err(BodyStop::Unsupported(
                        HtmlSemanticUnsupported::AmbiguousDocumentElement,
                    )));
                }
                html = Some((child, element));
            }
        }
        let Some((html, element)) = html else {
            return Ok(Ok(None));
        };
        let node = self.node(&html)?;
        if self.known(element)? != Some(KnownHtmlElement::Html) {
            return Ok(Ok(None));
        }
        self.charge(1)?;
        let mut children = match node.children() {
            HtmlCapturedChildren::Complete(children) => children,
            HtmlCapturedChildren::Unavailable => {
                return Ok(Err(BodyStop::Missing(
                    HtmlSemanticMissing::OrderedChildren(html),
                )));
            }
        };
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else {
                return Ok(Ok(None));
            };
            let node = self.node(&child)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && (self.known(element)? == Some(KnownHtmlElement::Body)
                    // The obsolete frameset spelling is outside the modern
                    // vocabulary; use the shared charged exact-unit comparator.
                    || (self.html_namespace(element)?
                        && self.control_equal_units(
                            element.name().local_name().code_units(),
                            &[102, 114, 97, 109, 101, 115, 101, 116],
                        )?))
            {
                return Ok(Ok(Some(child)));
            }
        }
    }
}
