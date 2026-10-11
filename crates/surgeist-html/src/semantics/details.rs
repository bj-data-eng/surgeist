//! Static details/summary relationships, HTML 877d6146487f §§4.11.1–2.
//! No fallback UI, activation, toggle events or exclusivity mutation.

use super::*;

/// Exact selected name and same-tree member subjects in ordinary tree order.
/// Membership includes the subject even when its name is absent or empty.
/// This does not certify name or group author conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDetailsNameGroup {
    name_attribute: Option<Attribute>,
    members: Vec<HtmlCaptureSubjectId>,
}
impl HtmlDetailsNameGroup {
    pub fn name_attribute(&self) -> Option<&Attribute> {
        self.name_attribute.as_ref()
    }
    pub fn members(&self) -> &[HtmlCaptureSubjectId] {
        &self.members
    }
    pub fn len(&self) -> usize {
        self.members.len()
    }
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}
impl<'a> HtmlSemanticSession<'a> {
    fn details_role(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        expected: KnownHtmlElement,
    ) -> Result<HtmlSemanticValue<()>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        Ok(if self.known(element)? == Some(expected) {
            HtmlSemanticValue::Known(())
        } else {
            HtmlSemanticValue::Inapplicable
        })
    }
    fn details_first_summary(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        let node = self.node(subject)?;
        self.charge(1)?;
        let mut children = match node.children() {
            HtmlCapturedChildren::Complete(children) => children,
            HtmlCapturedChildren::Unavailable => {
                return Ok(Err(HtmlSemanticMissing::OrderedChildren(subject.clone())));
            }
        };
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else {
                return Ok(Ok(None));
            };
            let node = self.node(&child)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Summary)
            {
                return Ok(Ok(Some(child)));
            }
        }
    }
    pub(super) fn details_summary(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        match self.details_role(subject, KnownHtmlElement::Details)? {
            HtmlSemanticValue::Known(()) => {}
            value => return Ok(value.map(|()| None)),
        }
        Ok(match self.details_first_summary(subject)? {
            Ok(value) => HtmlSemanticValue::Known(value),
            Err(missing) => HtmlSemanticValue::Missing(missing),
        })
    }
    pub(super) fn summary_for_parent_details(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        match self.details_role(subject, KnownHtmlElement::Summary)? {
            HtmlSemanticValue::Known(()) => {}
            value => return Ok(value.map(|()| false)),
        }
        let node = self.node(subject)?;
        let parent = match self.parent(node)? {
            Ok(Some(parent)) => parent,
            Ok(None) => return Ok(HtmlSemanticValue::Known(false)),
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        match self.details_role(&parent, KnownHtmlElement::Details)? {
            HtmlSemanticValue::Known(()) => {}
            _ => return Ok(HtmlSemanticValue::Known(false)),
        }
        Ok(match self.details_first_summary(&parent)? {
            Ok(first) => {
                self.charge(1)?;
                HtmlSemanticValue::Known(first.as_ref() == Some(subject))
            }
            Err(missing) => HtmlSemanticValue::Missing(missing),
        })
    }
    pub(super) fn details_name_group(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlDetailsNameGroup>, HtmlSemanticFailure> {
        match self.details_role(subject, KnownHtmlElement::Details)? {
            HtmlSemanticValue::Known(()) => {}
            HtmlSemanticValue::Inapplicable => return Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(value) => {
                return Ok(HtmlSemanticValue::Unsupported(value));
            }
            HtmlSemanticValue::Missing(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        }
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            unreachable!("details role established element");
        };
        let name = self.attribute(element, "name")?;
        self.interpreter
            .admit_attribute_raw(name.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let mut frames = Vec::new();
        let result = self.details_group_inner(subject, name, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }
    fn details_group_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        name: Option<&'a Attribute>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlDetailsNameGroup>, HtmlSemanticFailure> {
        let mut members = Vec::new();
        self.charge(1)?;
        if name.is_none_or(|a| a.value().is_empty()) {
            self.reserve_nested_output(&mut members, 1)?;
            self.charge(1)?;
            members.push(subject.clone());
        } else {
            let name_value = name.expect("nonempty selected name").value();
            let root = match self.ordinary_tree_root(subject)? {
                Ok(root) => root,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            let mut current = Some(root);
            while let Some(id) = current {
                let node = self.node(&id)?;
                if let HtmlCapturedNodeKind::Element(element) = node.kind()
                    && self.known(element)? == Some(KnownHtmlElement::Details)
                    && let Some(other) = self.attribute(element, "name")?
                {
                    self.charge(1)?;
                    if !other.value().is_empty() {
                        self.interpreter
                            .admit_attribute_raw(Some(other.value()))
                            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                        if self.control_equal(name_value, other.value())? {
                            self.reserve_nested_output(&mut members, 1)?;
                            self.charge(1)?;
                            members.push(id);
                        }
                    }
                }
                current = match self.ordinary_tree_next(node, frames)? {
                    Ok(current) => current,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
            }
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlDetailsNameGroup {
            name_attribute: name.cloned(),
            members,
        }))
    }
}
