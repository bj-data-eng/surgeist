//! Captured author constraints independent of intrinsic attribute content.
//! Ordinary ancestry, expanded names and shared token recognition supply actual
//! evidence. These operations perform no URL, activation, stylesheet or UI effect.
//! Work admits bounded intrinsic recognition and the allocation-free borrowed rel
//! scan, with polls between operations. The single output record is fallibly
//! admitted; retained capture handles share input backing outside variable-buffer
//! accounting. No parser instruction quantum is claimed.

use super::*;
use crate::attributes::stylesheet_link_context_work_bound;
use crate::{
    HtmlAttributeReport, HtmlBooleanAttribute, HtmlHyperlinkAttributeContext,
    HtmlImageControlsAttributeContext, HtmlImageMapAttributeContext, HtmlReferrerPolicy,
};

/// Ancestor evidence for a present img ismap, without URL interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlImageMapAncestorEvidence {
    AncestorAnchorWithHref {
        ancestor: HtmlCaptureSubjectId,
        href: Attribute,
    },
    NoAncestorAnchorWithHref,
    /// Intrinsic presence remains established while the author context is unavailable.
    Missing(HtmlSemanticMissing),
}

/// Only evidence actually consulted by the selected Boolean author-context query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlBooleanAuthorContextEvidence {
    NotRequired,
    ImageMap(HtmlImageMapAncestorEvidence),
    ImageControls { alt: Option<Attribute> },
    StylesheetLink { rel: Option<Attribute> },
}

/// Exact content report plus independent captured author-context evidence.
/// A missing ImageMap context does not erase the known Boolean content value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBooleanAuthorContextReadback {
    // One record, admitted before push. Construction couples report and evidence.
    data: Vec<BooleanAuthorContextData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct BooleanAuthorContextData {
    report: HtmlAttributeReport<bool>,
    evidence: HtmlBooleanAuthorContextEvidence,
}
impl HtmlBooleanAuthorContextReadback {
    pub fn attribute_report(&self) -> &HtmlAttributeReport<bool> {
        &self.data[0].report
    }
    pub fn evidence(&self) -> &HtmlBooleanAuthorContextEvidence {
        &self.data[0].evidence
    }
}

/// Captured a/area referrerpolicy content and its actual href-presence context.
/// The attribute report's context distinguishes observed missing href from an
/// absent referrerpolicy that required no href lookup. No request policy is resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlHyperlinkReferrerPolicyReadback {
    data: Vec<HyperlinkReferrerPolicyData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct HyperlinkReferrerPolicyData {
    report: HtmlAttributeReport<HtmlReferrerPolicy>,
    href: Option<Attribute>,
}
impl HtmlHyperlinkReferrerPolicyReadback {
    pub fn attribute_report(&self) -> &HtmlAttributeReport<HtmlReferrerPolicy> {
        &self.data[0].report
    }
    /// Actual href only when a present referrerpolicy required this observation.
    pub fn href_attribute(&self) -> Option<&Attribute> {
        self.data[0].href.as_ref()
    }
}

#[derive(Clone, Copy)]
enum BooleanContextKind {
    ImageMap,
    ImageControls,
    StylesheetDisabled,
}
impl BooleanContextKind {
    fn element(self) -> KnownHtmlElement {
        match self {
            Self::ImageMap | Self::ImageControls => KnownHtmlElement::Img,
            Self::StylesheetDisabled => KnownHtmlElement::Link,
        }
    }
    fn attribute(self) -> HtmlBooleanAttribute {
        match self {
            Self::ImageMap => HtmlBooleanAttribute::Ismap,
            Self::ImageControls => HtmlBooleanAttribute::ImageControls,
            Self::StylesheetDisabled => HtmlBooleanAttribute::StylesheetDisabled,
        }
    }
    fn local(self) -> &'static str {
        match self {
            Self::ImageMap => "ismap",
            Self::ImageControls => "controls",
            Self::StylesheetDisabled => "disabled",
        }
    }
}

impl HtmlSemanticSession<'_> {
    pub(super) fn image_map_attribute(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlBooleanAuthorContextReadback>, HtmlSemanticFailure> {
        self.boolean_author_context(subject, BooleanContextKind::ImageMap)
    }
    pub(super) fn image_controls_attribute(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlBooleanAuthorContextReadback>, HtmlSemanticFailure> {
        self.boolean_author_context(subject, BooleanContextKind::ImageControls)
    }
    pub(super) fn stylesheet_disabled_attribute(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlBooleanAuthorContextReadback>, HtmlSemanticFailure> {
        self.boolean_author_context(subject, BooleanContextKind::StylesheetDisabled)
    }
    fn boolean_author_context(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        kind: BooleanContextKind,
    ) -> Result<HtmlSemanticValue<HtmlBooleanAuthorContextReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(kind.element()) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, kind.local())?;
        self.admit_intrinsic(element, raw, 1)?;
        // Admit selected raw before source-context traversal; invalid spelling
        // retains true presence but a resource stop must not publish a prefix.
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let (report, evidence) = if raw.is_none() {
            let report = self
                .interpreter
                .boolean_attribute(kind.attribute(), element.name(), raw);
            (report, HtmlBooleanAuthorContextEvidence::NotRequired)
        } else {
            match kind {
                BooleanContextKind::ImageMap => {
                    let evidence = self.image_map_ancestor_evidence(subject)?;
                    let context = match &evidence {
                        HtmlImageMapAncestorEvidence::AncestorAnchorWithHref { .. } => {
                            HtmlImageMapAttributeContext::AncestorAnchorWithHref
                        }
                        HtmlImageMapAncestorEvidence::NoAncestorAnchorWithHref => {
                            HtmlImageMapAttributeContext::NoAncestorAnchorWithHref
                        }
                        HtmlImageMapAncestorEvidence::Missing(_) => {
                            HtmlImageMapAttributeContext::Unavailable
                        }
                    };
                    (
                        self.interpreter
                            .ismap_with_context(element.name(), raw, context),
                        HtmlBooleanAuthorContextEvidence::ImageMap(evidence),
                    )
                }
                BooleanContextKind::ImageControls => {
                    let alt = self.attribute(element, "alt")?;
                    self.charge(1)?;
                    let context = match alt {
                        None => HtmlImageControlsAttributeContext::AltMissing,
                        Some(a) if a.value().is_empty() => {
                            HtmlImageControlsAttributeContext::AltEmpty
                        }
                        Some(_) => HtmlImageControlsAttributeContext::AltNonempty,
                    };
                    (
                        self.interpreter
                            .image_controls_with_context(element.name(), raw, context),
                        HtmlBooleanAuthorContextEvidence::ImageControls { alt: alt.cloned() },
                    )
                }
                BooleanContextKind::StylesheetDisabled => {
                    let rel = self.attribute(element, "rel")?;
                    self.charge(1)?;
                    self.interpreter
                        .admit_attribute_raw(rel.map(Attribute::value))
                        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                    let work =
                        stylesheet_link_context_work_bound(rel.map_or(0, |a| a.value().len()))
                            .ok_or_else(|| {
                                self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
                            })?;
                    self.charge(work)?;
                    let context = self
                        .interpreter
                        .stylesheet_link_context(rel.map(Attribute::value))
                        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                    self.poll()?;
                    self.charge(1)?;
                    (
                        self.interpreter
                            .stylesheet_disabled(element.name(), raw, &context),
                        HtmlBooleanAuthorContextEvidence::StylesheetLink { rel: rel.cloned() },
                    )
                }
            }
        };
        let report = report.map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        self.poll()?;
        if report.intrinsic().is_none() {
            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
        }
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(BooleanAuthorContextData { report, evidence });
        Ok(HtmlSemanticValue::Known(HtmlBooleanAuthorContextReadback {
            data,
        }))
    }

    fn image_map_ancestor_evidence(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlImageMapAncestorEvidence, HtmlSemanticFailure> {
        let mut current = self.node(subject)?;
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                Ok(None) => return Ok(HtmlImageMapAncestorEvidence::NoAncestorAnchorWithHref),
                Err(missing) => return Ok(HtmlImageMapAncestorEvidence::Missing(missing)),
            };
            current = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = current.kind()
                && self.known(element)? == Some(KnownHtmlElement::A)
                && let Some(href) = self.attribute(element, "href")?
            {
                self.charge(1)?;
                return Ok(HtmlImageMapAncestorEvidence::AncestorAnchorWithHref {
                    ancestor: id,
                    href: href.clone(),
                });
            }
        }
    }

    pub(super) fn hyperlink_referrerpolicy_attribute(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlHyperlinkReferrerPolicyReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if !matches!(
            self.known(element)?,
            Some(KnownHtmlElement::A | KnownHtmlElement::Area)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "referrerpolicy")?;
        self.admit_intrinsic(element, raw, 9)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let (report, href) = if raw.is_some() {
            let href = self.attribute(element, "href")?;
            self.charge(1)?;
            let context = if href.is_some() {
                HtmlHyperlinkAttributeContext::WithHref
            } else {
                HtmlHyperlinkAttributeContext::WithoutHref
            };
            (
                self.interpreter
                    .referrerpolicy_for_hyperlink(element.name(), raw, context),
                href,
            )
        } else {
            (self.interpreter.referrerpolicy(element.name(), raw), None)
        };
        let report = report.map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        self.poll()?;
        if report.intrinsic().is_none() {
            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
        }
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(HyperlinkReferrerPolicyData {
            report,
            href: href.cloned(),
        });
        Ok(HtmlSemanticValue::Known(
            HtmlHyperlinkReferrerPolicyReadback { data },
        ))
    }
}
