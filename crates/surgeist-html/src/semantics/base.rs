//! HTML877d document base, base.href and target selection over one capture.
use super::*;
use crate::attributes::TargetRecovery;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentBaseUrl {
    value: HtmlBaseUrlRecord,
    base_element: Option<HtmlCaptureSubjectId>,
    frozen_revision: Option<HtmlBaseFactsRevision>,
}
impl HtmlDocumentBaseUrl {
    pub fn value(&self) -> &HtmlBaseUrlRecord {
        &self.value
    }
    pub fn base_element(&self) -> Option<&HtmlCaptureSubjectId> {
        self.base_element.as_ref()
    }
    pub fn frozen_revision(&self) -> Option<&HtmlBaseFactsRevision> {
        self.frozen_revision.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTargetOrigin {
    ExplicitOverride,
    OwnAttribute,
    DocumentBase(HtmlCaptureSubjectId),
    NoTarget,
    NoFormOwner,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlElementTarget {
    raw: Option<DocumentString>,
    value: Option<DocumentString>,
    origin: HtmlTargetOrigin,
}
impl HtmlElementTarget {
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw.as_ref()
    }
    pub fn value(&self) -> Option<&DocumentString> {
        self.value.as_ref()
    }
    pub fn origin(&self) -> &HtmlTargetOrigin {
        &self.origin
    }
}

/// The getter's parse is independent of CSP and the historical frozen slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBaseHref {
    raw: DocumentString,
    parsed: Option<HtmlBaseUrlRecord>,
}
impl HtmlBaseHref {
    pub fn raw(&self) -> &DocumentString {
        &self.raw
    }
    pub fn parsed(&self) -> Option<&HtmlBaseUrlRecord> {
        self.parsed.as_ref()
    }
    pub fn value(&self) -> &DocumentString {
        self.parsed
            .as_ref()
            .map_or(&self.raw, HtmlBaseUrlRecord::serialization)
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn base_fact(
        &mut self,
        node: HtmlCapturedNode<'a>,
        request: &HtmlSemanticRequest,
    ) -> Result<HtmlSemanticFact, HtmlSemanticFailure> {
        match request.query() {
            HtmlSemanticQuery::DocumentBaseUrl => Ok(HtmlSemanticFact::DocumentBaseUrl(
                if matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
                    self.document_base(node)?
                } else {
                    HtmlSemanticValue::Inapplicable
                },
            )),
            HtmlSemanticQuery::ElementTarget => {
                let target = match &request.operation {
                    SemanticOperation::TargetOverride(value) => Some(value),
                    SemanticOperation::Query(_)
                    | SemanticOperation::Noopener(_)
                    | SemanticOperation::DefaultFeed(_)
                    | SemanticOperation::DocumentIcon(_)
                    | SemanticOperation::NextDocumentIcon(_) => None,
                };
                Ok(HtmlSemanticFact::ElementTarget(
                    self.element_target(node, target)?,
                ))
            }
            HtmlSemanticQuery::BaseHref => Ok(HtmlSemanticFact::BaseHref(self.base_href(node)?)),
            HtmlSemanticQuery::FormSubmissionTarget => Ok(HtmlSemanticFact::FormSubmissionTarget(
                self.submission_target(node)?,
            )),
            _ => unreachable!("base_fact is dispatched only for the four base/target operations"),
        }
    }

    fn base_missing<T>(
        &mut self,
        subject: HtmlCaptureSubjectId,
        fact: HtmlBaseMissing,
    ) -> Result<HtmlSemanticValue<T>, HtmlSemanticFailure> {
        self.charge(1)?;
        Ok(HtmlSemanticValue::Missing(HtmlSemanticMissing::Base {
            subject,
            fact,
        }))
    }

    fn base_local(
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

    fn base_environment(
        &mut self,
        document: &crate::HtmlCaptureDocumentId,
    ) -> Result<Option<&'a crate::HtmlParseBaseFacts>, HtmlSemanticFailure> {
        let Some(facts) = &self.context.base_facts else {
            return Ok(None);
        };
        for row in &facts.documents {
            self.charge(1)?;
            if &row.document == document {
                return Ok(Some(&row.facts));
            }
        }
        Ok(None)
    }

    fn first_base(
        &mut self,
        document: HtmlCapturedNode<'a>,
        attribute: &str,
    ) -> Result<HtmlSemanticValue<Option<HtmlCapturedNode<'a>>>, HtmlSemanticFailure> {
        let mut frames = Vec::new();
        let result = (|| {
            let mut node = document;
            loop {
                if let HtmlCapturedNodeKind::Element(element) = node.kind()
                    && self.html_namespace(element)?
                    && self.base_local(element, b"base")?
                    && self.attribute(element, attribute)?.is_some()
                {
                    return Ok(HtmlSemanticValue::Known(Some(node)));
                }
                node = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(Some(id)) => self.node(&id)?,
                    Ok(None) => return Ok(HtmlSemanticValue::Known(None)),
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
            }
        })();
        self.release_scratch(frames.capacity());
        result
    }

    fn document_base(
        &mut self,
        document: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSemanticValue<HtmlDocumentBaseUrl>, HtmlSemanticFailure> {
        let first = match self.first_base(document, "href")? {
            HtmlSemanticValue::Known(first) => first,
            HtmlSemanticValue::Missing(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            _ => unreachable!("first_base returns a qualified selection or missing topology"),
        };
        if let Some(first) = first {
            let id = first.id();
            if let Some(facts) = &self.context.base_facts {
                for row in &facts.frozen {
                    self.charge(1)?;
                    if row.subject == id {
                        let Some(value) = &row.value else { break };
                        self.charge(3)?;
                        return Ok(HtmlSemanticValue::Known(HtmlDocumentBaseUrl {
                            value: value.clone(),
                            base_element: Some(id),
                            frozen_revision: Some(row.frozen_revision.clone()),
                        }));
                    }
                }
            }
            return self.base_missing(id, HtmlBaseMissing::FrozenBase);
        }
        self.charge(1)?;
        let document_id = document.node_document();
        let Some(facts) = self.base_environment(&document_id)? else {
            return self.base_missing(document.id(), HtmlBaseMissing::DocumentFacts);
        };
        match facts.document().fallback() {
            Ok(value) => {
                self.charge(1)?;
                Ok(HtmlSemanticValue::Known(HtmlDocumentBaseUrl {
                    value: value.clone(),
                    base_element: None,
                    frozen_revision: None,
                }))
            }
            Err(missing) => self.base_missing(document.id(), missing),
        }
    }

    fn parse_base_value(
        &mut self,
        subject: HtmlCaptureSubjectId,
        document: crate::HtmlCaptureDocumentId,
        raw: &DocumentString,
    ) -> Result<HtmlSemanticValue<&'a crate::HtmlBaseParseOutcome>, HtmlSemanticFailure> {
        let Some(facts) = self.base_environment(&document)? else {
            return self.base_missing(subject, HtmlBaseMissing::DocumentFacts);
        };
        let environment = facts.document();
        let fallback = match environment.fallback() {
            Ok(value) => value,
            Err(missing) => return self.base_missing(subject, missing),
        };
        let Some(encoding) = environment.encoding() else {
            return self.base_missing(subject, HtmlBaseMissing::CharacterEncoding);
        };
        for observation in facts.observations() {
            self.charge(1)?;
            if observation.inputs_match(fallback, encoding, environment.revision()) {
                self.charge(1)?;
                if observation.raw().len() != raw.len() {
                    continue;
                }
                let mut equal = true;
                for (a, b) in observation.raw().code_units().iter().zip(raw.code_units()) {
                    self.charge(1)?;
                    if a != b {
                        equal = false;
                        break;
                    }
                }
                if equal {
                    return Ok(HtmlSemanticValue::Known(observation.outcome()));
                }
            }
        }
        self.base_missing(subject, HtmlBaseMissing::ParseObservation)
    }

    fn base_href(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSemanticValue<HtmlBaseHref>, HtmlSemanticFailure> {
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? || !self.base_local(element, b"base")? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "href")?.map(Attribute::value);
        // The getter uses empty input for an absent href, unlike first-href selection.
        let raw = raw.unwrap_or(&self.context.empty_language);
        self.charge(1)?;
        let result = self.parse_base_value(node.id(), node.node_document(), raw)?;
        match result {
            HtmlSemanticValue::Known(outcome) => {
                self.charge(2)?;
                let parsed = match outcome {
                    crate::HtmlBaseParseOutcome::Failure => None,
                    crate::HtmlBaseParseOutcome::Parsed { record, .. } => Some(record.clone()),
                };
                Ok(HtmlSemanticValue::Known(HtmlBaseHref {
                    raw: raw.clone(),
                    parsed,
                }))
            }
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            _ => unreachable!("parse_base_value returns an observation or missing external facts"),
        }
    }

    fn target_result(
        &mut self,
        raw: Option<&DocumentString>,
        origin: HtmlTargetOrigin,
    ) -> Result<HtmlElementTarget, HtmlSemanticFailure> {
        let mut recovery = TargetRecovery::default();
        if let Some(raw) = raw {
            for (offset, unit) in raw.code_units().iter().enumerate() {
                self.charge(1)?;
                recovery.unit(offset, *unit);
            }
        }
        self.charge(2)?;
        let value = if recovery.dangerous().is_some() {
            let mut units = Vec::new();
            self.reserve_nested_output(&mut units, 6)?;
            for byte in b"_blank" {
                self.charge(1)?;
                units.push(u16::from(*byte));
            }
            Some(DocumentString::from_utf16(units))
        } else {
            raw.cloned()
        };
        Ok(HtmlElementTarget {
            raw: raw.cloned(),
            value,
            origin,
        })
    }

    fn element_target(
        &mut self,
        node: HtmlCapturedNode<'a>,
        explicit: Option<&DocumentString>,
    ) -> Result<HtmlSemanticValue<HtmlElementTarget>, HtmlSemanticFailure> {
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)?
            || !(self.base_local(element, b"a")?
                || self.base_local(element, b"area")?
                || self.base_local(element, b"form")?)
        {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        if let Some(raw) = explicit {
            return self
                .target_result(Some(raw), HtmlTargetOrigin::ExplicitOverride)
                .map(HtmlSemanticValue::Known);
        }
        if let Some(attribute) = self.attribute(element, "target")? {
            return self
                .target_result(Some(attribute.value()), HtmlTargetOrigin::OwnAttribute)
                .map(HtmlSemanticValue::Known);
        }
        self.charge(1)?;
        let document = node.node_document();
        let document = self.node(document.subject())?;
        match self.first_base(document, "target")? {
            HtmlSemanticValue::Known(Some(base)) => {
                let HtmlCapturedNodeKind::Element(element) = base.kind() else {
                    unreachable!()
                };
                let raw = self
                    .attribute(element, "target")?
                    .expect("selected base has immutable target")
                    .value();
                self.target_result(Some(raw), HtmlTargetOrigin::DocumentBase(base.id()))
                    .map(HtmlSemanticValue::Known)
            }
            HtmlSemanticValue::Known(None) => self
                .target_result(None, HtmlTargetOrigin::NoTarget)
                .map(HtmlSemanticValue::Known),
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            _ => unreachable!(),
        }
    }

    fn submission_target(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSemanticValue<HtmlElementTarget>, HtmlSemanticFailure> {
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let kind = self.known(element)?;
        if !matches!(
            kind,
            Some(KnownHtmlElement::Input | KnownHtmlElement::Button)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "formtarget")?;
        let override_value = if let Some(raw) = raw {
            match self
                .form_submitter_role(&node.id(), element, kind)?
                .submit_button
            {
                Ok(true) => Some(raw.value()),
                Ok(false) => None,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            }
        } else {
            None
        };
        match self.form_owner(&node.id())? {
            HtmlSemanticValue::Known(owner) => match owner.owner() {
                Some(form) => {
                    let form = self.node(form)?;
                    self.element_target(form, override_value)
                }
                None => self
                    .target_result(None, HtmlTargetOrigin::NoFormOwner)
                    .map(HtmlSemanticValue::Known),
            },
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(value) => Ok(HtmlSemanticValue::Unsupported(value)),
        }
    }
}
