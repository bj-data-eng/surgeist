//! HTML title.text and document.title, HTML 877d6146487f.
//! Child text uses the existing exact owner. Document readback then admits a
//! second real buffer for ASCII strip/collapse; both capacities are cumulative.

use super::*;

/// Normalized document title and the actual selected title element.
///
/// An empty selected title retains `Some(title_element)`. `None` proves absence,
/// including a complete SVG svg root with no direct SVG title. That SVG absence
/// fallback is the selected narrow reconciliation of HTML's omitted null case;
/// it does not select browser text extraction or canonicalization behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentTitle {
    value: DocumentString,
    title_element: Option<HtmlCaptureSubjectId>,
}
impl HtmlDocumentTitle {
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn title_element(&self) -> Option<&HtmlCaptureSubjectId> {
        self.title_element.as_ref()
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn title_text(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        if !self.title_local_is(element, b"title")? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.control_text_content(subject, HtmlControlTextOrigin::ChildText)
    }

    pub(super) fn document_title(
        &mut self,
        document: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSemanticValue<HtmlDocumentTitle>, HtmlSemanticFailure> {
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = document.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(document.id()),
            ));
        };
        let mut root = None;
        loop {
            self.charge(1)?;
            let Some(id) = children.next() else { break };
            let child = self.node(&id)?;
            if matches!(child.kind(), HtmlCapturedNodeKind::Element(_)) {
                self.charge(1)?;
                if root.is_some() {
                    return Ok(HtmlSemanticValue::Unsupported(
                        HtmlSemanticUnsupported::AmbiguousDocumentElement,
                    ));
                }
                root = Some(child);
            }
        }
        if let Some(root) = root {
            let HtmlCapturedNodeKind::Element(element) = root.kind() else {
                unreachable!()
            };
            if self.namespace_is(element, "http://www.w3.org/2000/svg")?
                && self.title_local_is(element, b"svg")?
            {
                self.charge(1)?;
                let HtmlCapturedChildren::Complete(mut children) = root.children() else {
                    return Ok(HtmlSemanticValue::Missing(
                        HtmlSemanticMissing::OrderedChildren(root.id()),
                    ));
                };
                loop {
                    self.charge(1)?;
                    let Some(id) = children.next() else {
                        return self.document_title_empty();
                    };
                    let child = self.node(&id)?;
                    if let HtmlCapturedNodeKind::Element(element) = child.kind()
                        && self.namespace_is(element, "http://www.w3.org/2000/svg")?
                        && self.title_local_is(element, b"title")?
                    {
                        return self.document_title_selected(id);
                    }
                }
            }
        }
        let mut frames = Vec::new();
        let result = self.document_title_html(document, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }

    fn document_title_html(
        &mut self,
        mut node: HtmlCapturedNode<'a>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlDocumentTitle>, HtmlSemanticFailure> {
        loop {
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.html_namespace(element)?
                && self.title_local_is(element, b"title")?
            {
                return self.document_title_selected(node.id());
            }
            let id = match self.ordinary_tree_next(node, frames)? {
                Ok(Some(id)) => id,
                Ok(None) => return self.document_title_empty(),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            node = self.node(&id)?;
        }
    }

    fn title_local_is(
        &mut self,
        element: &HtmlCapturedElement,
        expected: &[u8],
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let units = element.name().local_name().code_units();
        if units.len() != expected.len() {
            return Ok(false);
        }
        for (unit, byte) in units.iter().zip(expected) {
            self.charge(1)?;
            if *unit != u16::from(*byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn document_title_selected(
        &mut self,
        id: HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlDocumentTitle>, HtmlSemanticFailure> {
        let raw = match self.control_text_content(&id, HtmlControlTextOrigin::ChildText)? {
            HtmlSemanticValue::Known(raw) => raw,
            HtmlSemanticValue::Missing(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable | HtmlSemanticValue::Unsupported(_) => unreachable!(),
        };
        let mut units = Vec::new();
        let mut pending_space = false;
        for unit in raw.code_units() {
            self.charge(1)?;
            let unit = *unit;
            if matches!(unit, 9 | 10 | 12 | 13 | 32) {
                pending_space = !units.is_empty();
            } else {
                if pending_space {
                    self.reserve_nested_output(&mut units, 1)?;
                    self.charge(1)?;
                    units.push(32);
                    pending_space = false;
                }
                self.reserve_nested_output(&mut units, 1)?;
                self.charge(1)?;
                units.push(unit);
            }
        }
        self.charge(1)?;
        let value = if units.is_empty() {
            self.context.empty_language().clone()
        } else {
            // Actual admitted vector transfer. std Arc's fixed wrapper retains
            // the existing semantic getter's nonrecoverable allocation limits.
            DocumentString::from_utf16(units)
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlDocumentTitle {
            value,
            title_element: Some(id),
        }))
    }

    fn document_title_empty(
        &mut self,
    ) -> Result<HtmlSemanticValue<HtmlDocumentTitle>, HtmlSemanticFailure> {
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlDocumentTitle {
            value: self.context.empty_language().clone(),
            title_element: None,
        }))
    }
}
