//! HTML title.text and document.title, HTML 877d6146487f.
//! Child text uses the existing exact owner. Document readback then admits a
//! second real buffer for ASCII strip/collapse; both capacities are cumulative.

use super::*;
use ascii_whitespace::AsciiWhitespaceText;
use scan::name_part_is_with;
use surgeist_html::values::{DocumentString, HtmlNameInput};

/// Selects the ordinary Document preorder (HTML) or direct children of the
/// SVG document element (SVG) for the title lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDocumentTitleKind {
    Html,
    Svg,
}

impl HtmlDocumentTitleKind {
    /// Only the exact SVG `svg` expanded name selects the SVG branch.
    pub fn for_root<E>(
        name: &(impl HtmlNameInput + ?Sized),
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        if name_part_is_with(|| name.namespace_uri(), SVG_NAMESPACE, &mut step)?
            && name_part_is_with(|| Some(name.local_name()), b"svg", &mut step)?
        {
            Ok(Self::Svg)
        } else {
            Ok(Self::Html)
        }
    }

    /// Compare the reached Element's exact expanded name, charging the caller
    /// for each comparison. Prefixes are immaterial.
    pub fn matches<E>(
        self,
        name: &(impl HtmlNameInput + ?Sized),
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let namespace = match self {
            Self::Html => scan::HTML_NAMESPACE.as_bytes(),
            Self::Svg => SVG_NAMESPACE,
        };
        Ok(
            name_part_is_with(|| name.namespace_uri(), namespace, &mut step)?
                && name_part_is_with(|| Some(name.local_name()), b"title", &mut step)?,
        )
    }
}

const SVG_NAMESPACE: &[u8] = b"http://www.w3.org/2000/svg";

/// Strips and collapses ASCII whitespace from a stream of direct child Text.
/// Retain one value across the title's Text nodes. Other UTF-16 is unchanged.
#[derive(Default)]
pub struct HtmlDocumentTitleText {
    text: AsciiWhitespaceText,
}

impl HtmlDocumentTitleText {
    /// Feed one previously admitted input unit and consume the output in order.
    /// The caller admits and stores any output. Trailing whitespace has no output.
    pub fn push(&mut self, unit: u16) -> impl Iterator<Item = u16> + use<> {
        let (units, len) = self.text.push(unit);
        units.into_iter().take(len)
    }
}

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
        if !name_part_is_with(
            || Some(element.name().local_name()),
            b"title",
            || self.charge(1),
        )? {
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
            if let HtmlCapturedNodeKind::Element(element) = child.kind() {
                self.charge(1)?;
                if root.is_some() {
                    return Ok(HtmlSemanticValue::Unsupported(
                        HtmlSemanticUnsupported::AmbiguousDocumentElement,
                    ));
                }
                root = Some((child, element));
            }
        }
        if let Some((root, element)) = root
            && HtmlDocumentTitleKind::for_root(element.name(), || self.charge(1))?
                == HtmlDocumentTitleKind::Svg
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
                    && HtmlDocumentTitleKind::Svg.matches(element.name(), || self.charge(1))?
                {
                    return self.document_title_selected(id);
                }
            }
        }
        let mut frames = Vec::new();
        let result = self.document_title_html(document, &mut frames);
        self.release_scratch(&frames);
        result
    }

    fn document_title_html(
        &mut self,
        mut node: HtmlCapturedNode<'a>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlDocumentTitle>, HtmlSemanticFailure> {
        loop {
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && HtmlDocumentTitleKind::Html.matches(element.name(), || self.charge(1))?
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

    fn document_title_selected(
        &mut self,
        id: HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlDocumentTitle>, HtmlSemanticFailure> {
        let raw = match self.control_text_or_missing(&id, HtmlControlTextOrigin::ChildText)? {
            Ok(raw) => raw,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let mut units = Vec::new();
        let mut normalized = HtmlDocumentTitleText::default();
        for &unit in raw.code_units() {
            self.charge(1)?;
            for unit in normalized.push(unit) {
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
