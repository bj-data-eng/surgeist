//! HTML 877d #mq, link/style/source/meta surrounding rules over one capture.
//! No stylesheet, resource selection, rel classification or theme UI is executed.
use super::*;
use crate::{
    HtmlMediaAttributeReport, HtmlMediaCursorAction, HtmlMediaElementRole,
    HtmlMediaPlacementObservation, HtmlMediaValue, HtmlMediaValueCursor,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMediaGrammarEvidence {
    NotSpecified,
    External(HtmlMediaGrammarOutcome),
    Missing(HtmlSemanticMissing),
}
/// A necessary media condition, never a complete external application decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMediaCondition {
    DefaultAll,
    NoFilter,
    Unconditional,
    CssMatches,
    CssDoesNotMatch,
    Missing(HtmlSemanticMissing),
    Unsupported(HtmlMediaExternalCapability),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlHyperlinkMediaAdvice {
    condition: HtmlMediaCondition,
}
impl HtmlHyperlinkMediaAdvice {
    pub fn condition(&self) -> &HtmlMediaCondition {
        &self.condition
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlExternalResourceMediaGate {
    condition: HtmlMediaCondition,
}
impl HtmlExternalResourceMediaGate {
    pub fn condition(&self) -> &HtmlMediaCondition {
        &self.condition
    }
}
/// Conditional readers do not establish this link's actual rel relationships.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlLinkMediaRules {
    condition: HtmlMediaCondition,
}
impl HtmlLinkMediaRules {
    pub fn hyperlink_advisory(&self) -> HtmlHyperlinkMediaAdvice {
        HtmlHyperlinkMediaAdvice {
            condition: self.condition.clone(),
        }
    }
    pub fn external_resource_gate(&self) -> HtmlExternalResourceMediaGate {
        HtmlExternalResourceMediaGate {
            condition: self.condition.clone(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlStyleMediaGate {
    condition: HtmlMediaCondition,
}
impl HtmlStyleMediaGate {
    pub fn condition(&self) -> &HtmlMediaCondition {
        &self.condition
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSourceMediaContext {
    Picture,
    MediaElement,
    Other,
    Missing(HtmlSemanticMissing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSourceMediaDecision {
    NoFilter,
    NoMediaExclusion(HtmlMediaCondition),
    Skip,
    Missing(HtmlSemanticMissing),
    Unsupported(HtmlMediaExternalCapability),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlPictureSourceRequirement {
    SatisfiedByTypePresence,
    SatisfiedByRestrictiveMedia,
    NotApplicable,
    NotTriggered,
    Violated { following: HtmlCaptureSubjectId },
    Missing(HtmlSemanticMissing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlThemeColorMediaUniqueness {
    NotRequired,
    NoDocumentMetadataScope,
    NoHtmlDocumentScope,
    Unique {
        document: HtmlCaptureSubjectId,
    },
    Duplicate {
        document: HtmlCaptureSubjectId,
        peer: HtmlCaptureSubjectId,
    },
    Missing(HtmlSemanticMissing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaMediaProcessing {
    NoProcessingEffect,
    ThemeColor {
        condition: HtmlMediaCondition,
        uniqueness: HtmlThemeColorMediaUniqueness,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMediaProcessing {
    Link(HtmlLinkMediaRules),
    Style(HtmlStyleMediaGate),
    Source {
        context: HtmlSourceMediaContext,
        decision: HtmlSourceMediaDecision,
        type_attribute: Option<Attribute>,
        picture_requirement: HtmlPictureSourceRequirement,
    },
    Meta(HtmlMetaMediaProcessing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaAttributeReadback {
    data: Vec<MediaReadbackData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct MediaReadbackData {
    local: HtmlMediaAttributeReport,
    grammar: HtmlMediaGrammarEvidence,
    processing: HtmlMediaProcessing,
}
impl HtmlMediaAttributeReadback {
    pub(super) fn owned_backing(&self) -> (usize, usize) {
        (
            self.data.capacity(),
            self.data.capacity() * std::mem::size_of::<MediaReadbackData>(),
        )
    }
    pub fn local(&self) -> &HtmlMediaAttributeReport {
        &self.data[0].local
    }
    pub fn grammar(&self) -> &HtmlMediaGrammarEvidence {
        &self.data[0].grammar
    }
    pub fn processing(&self) -> &HtmlMediaProcessing {
        &self.data[0].processing
    }
}

impl<'a> HtmlSemanticSession<'a> {
    fn media_raw(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        let a = self.attribute(element, local)?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(a.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        Ok(a)
    }
    fn media_value(
        &mut self,
        raw: Option<&Attribute>,
    ) -> Result<HtmlMediaValue, HtmlSemanticFailure> {
        let mut cursor = HtmlMediaValueCursor::new(raw.map(|a| a.value().code_units()));
        while cursor.next_action() != HtmlMediaCursorAction::Complete {
            // Finish includes a fixed three-unit keyword comparison.
            self.charge(if cursor.next_action() == HtmlMediaCursorAction::Finish {
                4
            } else {
                1
            })?;
            cursor.advance();
        }
        Ok(cursor.result().expect("completed admitted scan"))
    }
    fn media_external(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a HtmlMediaFact>, HtmlSemanticFailure> {
        let Some(facts) = self.context.media.as_ref() else {
            return Ok(None);
        };
        for fact in facts.facts() {
            self.charge(1)?;
            if fact.input().subject() == subject {
                return Ok(Some(fact));
            }
        }
        Ok(None)
    }
    pub(super) fn media_attribute_rules(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMediaAttributeReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        // This query's original field admission precedes role/namespace shortcuts.
        // The dispatcher defers classification only for this owning query.
        let raw = self.media_raw(element, "media")?;
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let Some(role) = HtmlMediaElementRole::from_known(self.known(element)?) else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let name = if role == HtmlMediaElementRole::Meta {
            self.media_raw(element, "name")?
        } else {
            None
        };
        let value = self.media_value(raw)?;
        let external = if raw.is_some() {
            self.media_external(subject)?
        } else {
            None
        };
        self.charge(1)?;
        let grammar = if raw.is_none() {
            HtmlMediaGrammarEvidence::NotSpecified
        } else {
            match external
                .and_then(HtmlMediaFact::grammar)
                .map(HtmlMediaGrammarAttestation::outcome)
            {
                None | Some(HtmlMediaGrammarOutcome::Unavailable) => {
                    HtmlMediaGrammarEvidence::Missing(HtmlSemanticMissing::MediaGrammar(
                        subject.clone(),
                    ))
                }
                Some(outcome) => HtmlMediaGrammarEvidence::External(outcome),
            }
        };
        let condition = if raw.is_none() {
            if matches!(
                role,
                HtmlMediaElementRole::Link | HtmlMediaElementRole::Style
            ) {
                HtmlMediaCondition::DefaultAll
            } else {
                HtmlMediaCondition::NoFilter
            }
        } else if value.matches_unconditionally() {
            HtmlMediaCondition::Unconditional
        } else {
            match external
                .and_then(HtmlMediaFact::matching)
                .map(HtmlMediaMatchAttestation::outcome)
            {
                Some(HtmlMediaMatchOutcome::Matches) => HtmlMediaCondition::CssMatches,
                Some(HtmlMediaMatchOutcome::DoesNotMatch) => HtmlMediaCondition::CssDoesNotMatch,
                None | Some(HtmlMediaMatchOutcome::Unavailable) => HtmlMediaCondition::Missing(
                    HtmlSemanticMissing::MediaEnvironment(subject.clone()),
                ),
                Some(HtmlMediaMatchOutcome::Unsupported(reason)) => {
                    HtmlMediaCondition::Unsupported(reason)
                }
            }
        };
        let mut frames = Vec::new();
        let result = (|| {
            self.charge(1)?;
            let (placement, processing) = match role {
                HtmlMediaElementRole::Link => (
                    HtmlMediaPlacementObservation::Unapplied,
                    HtmlMediaProcessing::Link(HtmlLinkMediaRules { condition }),
                ),
                HtmlMediaElementRole::Style => (
                    HtmlMediaPlacementObservation::Unapplied,
                    HtmlMediaProcessing::Style(HtmlStyleMediaGate { condition }),
                ),
                HtmlMediaElementRole::Source => {
                    let type_attribute = self.media_raw(element, "type")?;
                    let context = self.source_media_context(node)?;
                    let picture_requirement = self.picture_media_requirement(
                        subject,
                        node,
                        &context,
                        type_attribute.is_some(),
                        value,
                    )?;
                    let decision = match condition {
                        HtmlMediaCondition::NoFilter => HtmlSourceMediaDecision::NoFilter,
                        HtmlMediaCondition::CssDoesNotMatch => HtmlSourceMediaDecision::Skip,
                        HtmlMediaCondition::Missing(gap) => HtmlSourceMediaDecision::Missing(gap),
                        HtmlMediaCondition::Unsupported(reason) => {
                            HtmlSourceMediaDecision::Unsupported(reason)
                        }
                        other => HtmlSourceMediaDecision::NoMediaExclusion(other),
                    };
                    (
                        HtmlMediaPlacementObservation::Unapplied,
                        HtmlMediaProcessing::Source {
                            context,
                            decision,
                            type_attribute: type_attribute.cloned(),
                            picture_requirement,
                        },
                    )
                }
                HtmlMediaElementRole::Meta => {
                    let theme = self.meta_author_theme_color(name.map(Attribute::value))?;
                    let placement = if raw.is_none() {
                        HtmlMetaMediaPlacement::NotSpecified
                    } else if theme {
                        HtmlMetaMediaPlacement::AllowedForThemeColor
                    } else {
                        HtmlMetaMediaPlacement::Forbidden
                    };
                    let processing = if theme {
                        let uniqueness = if let Some(raw) = raw {
                            self.theme_media_uniqueness(subject, raw.value(), &mut frames)?
                        } else {
                            HtmlThemeColorMediaUniqueness::NotRequired
                        };
                        HtmlMetaMediaProcessing::ThemeColor {
                            condition,
                            uniqueness,
                        }
                    } else {
                        HtmlMetaMediaProcessing::NoProcessingEffect
                    };
                    (
                        HtmlMediaPlacementObservation::Known(placement),
                        HtmlMediaProcessing::Meta(processing),
                    )
                }
            };
            let mut data = Vec::new();
            self.reserve_nested_output(&mut data, 1)?;
            self.charge(1)?;
            data.push(MediaReadbackData {
                local: HtmlMediaAttributeReport::observed(raw, name, role, value, placement),
                grammar,
                processing,
            });
            Ok(HtmlSemanticValue::Known(HtmlMediaAttributeReadback {
                data,
            }))
        })();
        self.release_scratch(frames.capacity());
        result
    }
    fn source_media_context(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSourceMediaContext, HtmlSemanticFailure> {
        let parent = match self.parent(node)? {
            Ok(Some(id)) => self.node(&id)?,
            Ok(None) => return Ok(HtmlSourceMediaContext::Other),
            Err(gap) => return Ok(HtmlSourceMediaContext::Missing(gap)),
        };
        let HtmlCapturedNodeKind::Element(e) = parent.kind() else {
            return Ok(HtmlSourceMediaContext::Other);
        };
        Ok(match self.known(e)? {
            Some(KnownHtmlElement::Picture) => HtmlSourceMediaContext::Picture,
            Some(KnownHtmlElement::Audio | KnownHtmlElement::Video) => {
                HtmlSourceMediaContext::MediaElement
            }
            _ => HtmlSourceMediaContext::Other,
        })
    }
    fn picture_media_requirement(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        node: HtmlCapturedNode<'a>,
        context: &HtmlSourceMediaContext,
        type_present: bool,
        value: HtmlMediaValue,
    ) -> Result<HtmlPictureSourceRequirement, HtmlSemanticFailure> {
        self.charge(1)?;
        if type_present {
            return Ok(HtmlPictureSourceRequirement::SatisfiedByTypePresence);
        }
        if value.restrictive_picture_value() {
            return Ok(HtmlPictureSourceRequirement::SatisfiedByRestrictiveMedia);
        }
        match context {
            HtmlSourceMediaContext::Missing(gap) => {
                return Ok(HtmlPictureSourceRequirement::Missing(gap.clone()));
            }
            HtmlSourceMediaContext::Picture => {}
            _ => return Ok(HtmlPictureSourceRequirement::NotApplicable),
        }
        let parent = match self.parent(node)? {
            Ok(Some(id)) => self.node(&id)?,
            _ => unreachable!("known picture ordinary parent"),
        };
        let children = match parent.children() {
            HtmlCapturedChildren::Complete(children) => children,
            HtmlCapturedChildren::Unavailable => {
                return Ok(HtmlPictureSourceRequirement::Missing(
                    HtmlSemanticMissing::OrderedChildren(parent.id()),
                ));
            }
        };
        let mut after = false;
        for id in children {
            self.charge(1)?;
            if &id == subject {
                after = true;
                continue;
            }
            if !after {
                continue;
            }
            let sibling = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = sibling.kind() {
                let trigger = match self.known(e)? {
                    Some(KnownHtmlElement::Source) => true,
                    Some(KnownHtmlElement::Img) => self.attribute(e, "srcset")?.is_some(),
                    _ => false,
                };
                if trigger {
                    return Ok(HtmlPictureSourceRequirement::Violated { following: id });
                }
            }
        }
        Ok(HtmlPictureSourceRequirement::NotTriggered)
    }
    fn theme_media_uniqueness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        raw: &DocumentString,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlThemeColorMediaUniqueness, HtmlSemanticFailure> {
        let root = match self.ordinary_tree_root(subject)? {
            Ok(root) => root,
            Err(gap) => return Ok(HtmlThemeColorMediaUniqueness::Missing(gap)),
        };
        let root_node = self.node(&root)?;
        self.charge(1)?;
        let HtmlCapturedNodeKind::Document(document) = root_node.kind() else {
            return Ok(HtmlThemeColorMediaUniqueness::NoDocumentMetadataScope);
        };
        match document.document_class() {
            HtmlDocumentClass::Html => {}
            HtmlDocumentClass::NonHtml => {
                return Ok(HtmlThemeColorMediaUniqueness::NoHtmlDocumentScope);
            }
            HtmlDocumentClass::Unavailable => {
                return Ok(HtmlThemeColorMediaUniqueness::Missing(
                    HtmlSemanticMissing::DocumentClass(root),
                ));
            }
        }
        let mut missing = None;
        let mut current = Some(root.clone());
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(e) = node.kind()
                && self.known(e)? == Some(KnownHtmlElement::Meta)
            {
                let name = self.media_raw(e, "name")?;
                if self.meta_author_theme_color(name.map(Attribute::value))?
                    && let Some(peer) = self.media_raw(e, "media")?
                {
                    self.charge(1)?;
                    if peer.value().len() == raw.len() {
                        let mut equal = true;
                        for (a, b) in peer.value().code_units().iter().zip(raw.code_units()) {
                            self.charge(1)?;
                            if a != b {
                                equal = false;
                                break;
                            }
                        }
                        if equal {
                            return Ok(HtmlThemeColorMediaUniqueness::Duplicate {
                                document: root,
                                peer: id,
                            });
                        }
                    }
                }
            }
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(next) => next,
                Err(gap) => {
                    missing.get_or_insert(gap);
                    self.ordinary_tree_resume(frames)?
                }
            };
        }
        Ok(missing.map_or(
            HtmlThemeColorMediaUniqueness::Unique { document: root },
            HtmlThemeColorMediaUniqueness::Missing,
        ))
    }
}
