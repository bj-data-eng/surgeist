//! Static details/summary relationships, HTML 877d6146487f §§4.11.1–2.
//! No fallback UI, activation, toggle events or exclusivity mutation.

use super::*;
use surgeist_html::{values::Attribute, vocabulary::KnownHtmlElement};

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
    /// The element when it is the `expected` HTML element, and otherwise the value that answers
    /// the query without further reading.
    fn details_role<T>(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        expected: KnownHtmlElement,
    ) -> Result<Result<&'a HtmlCapturedElement, HtmlSemanticValue<T>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(Err(HtmlSemanticValue::Inapplicable));
        };
        if !self.html_namespace(element)? {
            return Ok(Err(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            )));
        }
        Ok(if self.known(element)? == Some(expected) {
            Ok(element)
        } else {
            Err(HtmlSemanticValue::Inapplicable)
        })
    }
    pub(super) fn details_summary(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        if let Err(value) = self.details_role(subject, KnownHtmlElement::Details)? {
            return Ok(value);
        }
        Ok(
            match self.first_html_element_child(subject, KnownHtmlElement::Summary)? {
                Ok(value) => HtmlSemanticValue::Known(value),
                Err(missing) => HtmlSemanticValue::Missing(missing),
            },
        )
    }
    pub(super) fn summary_for_parent_details(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        if let Err(value) = self.details_role(subject, KnownHtmlElement::Summary)? {
            return Ok(value);
        }
        let node = self.node(subject)?;
        let parent = match self.parent(node)? {
            Ok(Some(parent)) => parent,
            Ok(None) => return Ok(HtmlSemanticValue::Known(false)),
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        if self
            .details_role::<bool>(&parent, KnownHtmlElement::Details)?
            .is_err()
        {
            return Ok(HtmlSemanticValue::Known(false));
        }
        Ok(
            match self.first_html_element_child(&parent, KnownHtmlElement::Summary)? {
                Ok(first) => {
                    self.charge(1)?;
                    HtmlSemanticValue::Known(first.as_ref() == Some(subject))
                }
                Err(missing) => HtmlSemanticValue::Missing(missing),
            },
        )
    }
    pub(super) fn details_name_group(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlDetailsNameGroup>, HtmlSemanticFailure> {
        let element = match self.details_role(subject, KnownHtmlElement::Details)? {
            Ok(element) => element,
            Err(value) => return Ok(value),
        };
        // The element is read once more, and charged once more.
        self.charge(1)?;
        let name = self.attribute(element, "name")?;
        self.interpreter
            .admit_attribute_raw(name.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let mut frames = Vec::new();
        let result = self.details_group_inner(subject, name, &mut frames);
        self.release_scratch(&frames);
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
        if let Some(name_value) = name.map(Attribute::value).filter(|value| !value.is_empty()) {
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
        } else {
            self.reserve_nested_output(&mut members, 1)?;
            self.charge(1)?;
            members.push(subject.clone());
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlDetailsNameGroup {
            name_attribute: name.cloned(),
            members,
        }))
    }
}
