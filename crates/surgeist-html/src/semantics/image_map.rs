//! Immutable map area membership, HTML 877d6146487f. No geometry or activation.

use super::*;

/// Source-ordered HTML area subjects from one qualified immutable capture.
/// This is not a live HTMLCollection or a named lookup/image-map association.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaCollection {
    areas: Vec<HtmlCaptureSubjectId>,
}
impl HtmlAreaCollection {
    pub fn areas(&self) -> &[HtmlCaptureSubjectId] {
        &self.areas
    }
    pub fn len(&self) -> usize {
        self.areas.len()
    }
    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn map_areas(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlAreaCollection>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Map) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut frames = Vec::new();
        let result = self.map_areas_inner(node, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }
    fn map_areas_inner(
        &mut self,
        root: HtmlCapturedNode<'a>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlAreaCollection>, HtmlSemanticFailure> {
        let mut areas = Vec::new();
        let mut current = match self.ordinary_tree_next(root, frames)? {
            Ok(current) => current,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        while let Some(id) = current {
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Area)
            {
                self.reserve_nested_output(&mut areas, 1)?;
                self.charge(1)?;
                areas.push(id);
            }
            // Membership does not prune nested maps, foreign wrappers, areas,
            // or any other ordinary element subtree.
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(current) => current,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlAreaCollection { areas }))
    }
}
