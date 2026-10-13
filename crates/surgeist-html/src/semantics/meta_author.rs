//! HTML 877d #the-meta-element: independent captured author structure.
//! This does not establish complete meta conformance, encoding, CSS or pragma effects.

use super::*;

/// Exactly one discriminator must be specified, regardless of its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaDiscriminatorRequirement {
    Satisfied,
    NoneSpecified,
    MultipleSpecified,
}

/// Content presence is independent of discriminator exclusivity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaContentPresenceRequirement {
    Satisfied,
    MissingRequiredContent,
    UnexpectedContent,
}

/// At most one HTML meta with a specified charset per actual ordinary Document.
/// This is not the broader meta-based encoding-declaration/serialization proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaCharsetUniqueness {
    NotRequired,
    NoDocumentMetadataScope,
    Unique,
    Duplicate(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}

/// Exact own attributes and independent source-defined structural observations.
/// Known delivery never certifies full element or document conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMetaAttributeAuthorStructureReport {
    data: Vec<MetaAuthorData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MetaAuthorData {
    name: Option<Attribute>,
    http_equiv: Option<Attribute>,
    charset: Option<Attribute>,
    itemprop: Option<Attribute>,
    content: Option<Attribute>,
    media: Option<Attribute>,
    discriminator: HtmlMetaDiscriminatorRequirement,
    content_presence: HtmlMetaContentPresenceRequirement,
    media_placement: HtmlMetaMediaPlacement,
    charset_uniqueness: HtmlMetaCharsetUniqueness,
}

impl HtmlMetaAttributeAuthorStructureReport {
    pub fn name_attribute(&self) -> Option<&Attribute> {
        self.data[0].name.as_ref()
    }
    pub fn http_equiv_attribute(&self) -> Option<&Attribute> {
        self.data[0].http_equiv.as_ref()
    }
    pub fn charset_attribute(&self) -> Option<&Attribute> {
        self.data[0].charset.as_ref()
    }
    pub fn itemprop_attribute(&self) -> Option<&Attribute> {
        self.data[0].itemprop.as_ref()
    }
    pub fn content_attribute(&self) -> Option<&Attribute> {
        self.data[0].content.as_ref()
    }
    pub fn media_attribute(&self) -> Option<&Attribute> {
        self.data[0].media.as_ref()
    }
    pub fn discriminator_requirement(&self) -> HtmlMetaDiscriminatorRequirement {
        self.data[0].discriminator
    }
    pub fn content_presence_requirement(&self) -> HtmlMetaContentPresenceRequirement {
        self.data[0].content_presence
    }
    pub fn media_placement(&self) -> HtmlMetaMediaPlacement {
        self.data[0].media_placement
    }
    pub fn charset_uniqueness(&self) -> &HtmlMetaCharsetUniqueness {
        &self.data[0].charset_uniqueness
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn meta_attribute_author_structure(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMetaAttributeAuthorStructureReport>, HtmlSemanticFailure>
    {
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
        let name = self.meta_author_attribute(element, "name")?;
        let http_equiv = self.meta_author_attribute(element, "http-equiv")?;
        let charset = self.meta_author_attribute(element, "charset")?;
        let itemprop = self.meta_author_attribute(element, "itemprop")?;
        let content = self.meta_author_attribute(element, "content")?;
        let media = self.meta_author_attribute(element, "media")?;
        self.charge(1)?;
        let count = usize::from(name.is_some())
            + usize::from(http_equiv.is_some())
            + usize::from(charset.is_some())
            + usize::from(itemprop.is_some());
        let discriminator = match count {
            0 => HtmlMetaDiscriminatorRequirement::NoneSpecified,
            1 => HtmlMetaDiscriminatorRequirement::Satisfied,
            _ => HtmlMetaDiscriminatorRequirement::MultipleSpecified,
        };
        self.charge(1)?;
        let required = name.is_some() || http_equiv.is_some() || itemprop.is_some();
        let content_presence = match (required, content.is_some()) {
            (true, false) => HtmlMetaContentPresenceRequirement::MissingRequiredContent,
            (false, true) => HtmlMetaContentPresenceRequirement::UnexpectedContent,
            _ => HtmlMetaContentPresenceRequirement::Satisfied,
        };
        self.charge(1)?;
        let media_placement = if media.is_none() {
            HtmlMetaMediaPlacement::NotSpecified
        } else if self.meta_author_theme_color(name.map(Attribute::value))? {
            HtmlMetaMediaPlacement::AllowedForThemeColor
        } else {
            HtmlMetaMediaPlacement::Forbidden
        };
        let mut frames = Vec::new();
        let result = if charset.is_some() {
            self.meta_charset_uniqueness(subject, &mut frames)
        } else {
            self.charge(1)
                .map(|()| HtmlMetaCharsetUniqueness::NotRequired)
        };
        self.release_scratch(frames.capacity());
        let charset_uniqueness = result?;
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(MetaAuthorData {
            name: name.cloned(),
            http_equiv: http_equiv.cloned(),
            charset: charset.cloned(),
            itemprop: itemprop.cloned(),
            content: content.cloned(),
            media: media.cloned(),
            discriminator,
            content_presence,
            media_placement,
            charset_uniqueness,
        });
        Ok(HtmlSemanticValue::Known(
            HtmlMetaAttributeAuthorStructureReport { data },
        ))
    }

    fn meta_author_attribute(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        let attribute = self.attribute(element, local)?;
        // Every own retained influencing raw value crosses the existing original
        // UTF-16 admission boundary before inspection or delivery.
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(attribute.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        Ok(attribute)
    }

    pub(super) fn meta_author_theme_color(
        &mut self,
        name: Option<&DocumentString>,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let mut cursor =
            crate::media::ThemeColorNameCursor::new(name.map(DocumentString::code_units));
        while !cursor.is_complete() {
            self.charge(1)?;
            cursor.advance();
        }
        Ok(cursor.result())
    }

    fn meta_charset_uniqueness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlMetaCharsetUniqueness, HtmlSemanticFailure> {
        let root = match self.ordinary_tree_root(subject)? {
            Ok(root) => root,
            Err(missing) => return Ok(HtmlMetaCharsetUniqueness::Missing(missing)),
        };
        let root_node = self.node(&root)?;
        self.charge(1)?;
        if !matches!(root_node.kind(), HtmlCapturedNodeKind::Document(_)) {
            return Ok(HtmlMetaCharsetUniqueness::NoDocumentMetadataScope);
        }
        let mut missing = None;
        let mut current = Some(root);
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Meta)
                && self.attribute(element, "charset")?.is_some()
            {
                // Source counts specified attributes, not valid declarations.
                // No competitor value is read, retained or admitted.
                self.charge(1)?;
                return Ok(HtmlMetaCharsetUniqueness::Duplicate(id));
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
            HtmlMetaCharsetUniqueness::Unique,
            HtmlMetaCharsetUniqueness::Missing,
        ))
    }
}
