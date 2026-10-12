//! HTML 877d #meta-description: content presence and per-Document uniqueness.
//! Raw metadata, author requirements and contextual evidence remain independent.

use super::*;

/// The selected name requires content; a specified empty content satisfies this
/// presence rule. This is not an assessment of the description's meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaDescriptionContentRequirement {
    Satisfied,
    MissingContent,
}

/// Per-Document description uniqueness, independent of content presence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaDescriptionUniqueness {
    /// The established ordinary root is not a Document.
    NoDocumentMetadataScope,
    Unique,
    Duplicate(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}

/// Actual metadata pair and independent bounded author observations.
/// Known delivery is not complete meta or document conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMetaDescriptionAuthorReport {
    data: Vec<MetaDescriptionData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct MetaDescriptionData {
    name: Attribute,
    content: Option<Attribute>,
    effective_content: DocumentString,
    uniqueness: HtmlMetaDescriptionUniqueness,
}
impl HtmlMetaDescriptionAuthorReport {
    pub fn name_attribute(&self) -> &Attribute {
        &self.data[0].name
    }
    pub fn content_attribute(&self) -> Option<&Attribute> {
        self.data[0].content.as_ref()
    }
    /// Missing content supplies an effective empty value without becoming present.
    pub fn effective_content(&self) -> &DocumentString {
        &self.data[0].effective_content
    }
    pub fn content_requirement(&self) -> HtmlMetaDescriptionContentRequirement {
        if self.data[0].content.is_some() {
            HtmlMetaDescriptionContentRequirement::Satisfied
        } else {
            HtmlMetaDescriptionContentRequirement::MissingContent
        }
    }
    /// Whether prose describes this page appropriately is a human semantic author
    /// obligation. No lexical validity or overall conformance is inferred here.
    pub fn content_appropriateness(&self) -> crate::HtmlAttributeContextAuthorState {
        crate::HtmlAttributeContextAuthorState::Unchecked
    }
    pub fn uniqueness(&self) -> &HtmlMetaDescriptionUniqueness {
        &self.data[0].uniqueness
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn meta_description_author(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMetaDescriptionAuthorReport>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Meta) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let Some(name) = self.meta_description_name(element)? else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let content = self.attribute(element, "content")?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(content.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let mut frames = Vec::new();
        let result = self.meta_description_uniqueness(subject, &mut frames);
        self.release_scratch(frames.capacity());
        let uniqueness = result?;
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(MetaDescriptionData {
            name: name.clone(),
            content: content.cloned(),
            effective_content: content.map_or_else(
                || self.context.empty_language().clone(),
                |a| a.value().clone(),
            ),
            uniqueness,
        });
        Ok(HtmlSemanticValue::Known(HtmlMetaDescriptionAuthorReport {
            data,
        }))
    }

    fn meta_description_name(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        let Some(name) = self.attribute(element, "name")? else {
            return Ok(None);
        };
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(Some(name.value()))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        self.charge(1)?;
        let units = name.value().code_units();
        if units.len() != 11 {
            return Ok(None);
        }
        // A fixed ASCII keyword proof: no trim, Unicode fold or allocated copy.
        for (unit, byte) in units.iter().zip(b"description") {
            self.charge(1)?;
            if *unit > 127 || (*unit as u8).to_ascii_lowercase() != *byte {
                return Ok(None);
            }
        }
        Ok(Some(name))
    }

    fn meta_description_uniqueness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlMetaDescriptionUniqueness, HtmlSemanticFailure> {
        let root = match self.ordinary_tree_root(subject)? {
            Ok(root) => root,
            Err(missing) => return Ok(HtmlMetaDescriptionUniqueness::Missing(missing)),
        };
        let root_node = self.node(&root)?;
        self.charge(1)?;
        if !matches!(root_node.kind(), HtmlCapturedNodeKind::Document(_)) {
            return Ok(HtmlMetaDescriptionUniqueness::NoDocumentMetadataScope);
        }
        let mut missing = None;
        let mut current = Some(root);
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Meta)
                && self.meta_description_name(element)?.is_some()
            {
                // Missing content or unrelated author violations do not remove
                // this actual matching meta from the source uniqueness count.
                self.charge(1)?;
                return Ok(HtmlMetaDescriptionUniqueness::Duplicate(id));
            }
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(next) => next,
                Err(gap) => {
                    if missing.is_none() {
                        self.charge(1)?;
                        missing = Some(gap);
                    }
                    self.ordinary_tree_resume(frames)?
                }
            };
        }
        self.charge(1)?;
        Ok(missing.map_or(
            HtmlMetaDescriptionUniqueness::Unique,
            HtmlMetaDescriptionUniqueness::Missing,
        ))
    }
}
