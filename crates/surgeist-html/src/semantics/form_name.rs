//! Independent local and actual Document forms-collection name observations.
//! [HTML 877d form name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-form-name).
//! No live collection, named-property lookup, reflection or overall conformance.

use super::*;
use crate::{HtmlAttributeReport, HtmlFormName};

/// Actual specified-value uniqueness, independent of the local nonempty rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlFormNameUniqueness {
    NoSpecifiedName,
    /// Established ordinary root is not a Document; there is no forms collection.
    NoFormsCollection,
    Unique,
    Duplicate(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}
/// One admitted immutable record. Inner Missing/local failure is not conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlFormNameAuthorReport {
    data: Vec<FormNameData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct FormNameData {
    local: HtmlAttributeReport<HtmlFormName>,
    uniqueness: HtmlFormNameUniqueness,
}
impl HtmlFormNameAuthorReport {
    pub fn local(&self) -> &HtmlAttributeReport<HtmlFormName> {
        &self.data[0].local
    }
    pub fn uniqueness(&self) -> &HtmlFormNameUniqueness {
        &self.data[0].uniqueness
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn form_name_author(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlFormNameAuthorReport>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Form) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "name")?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let work = HtmlValueInterpreter::form_name_report_work_bound(element.name(), raw)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)?;
        let local = self
            .interpreter
            .form_name(element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        self.poll()?;
        // HTML's local nonempty requirement and collection uniqueness are
        // independent. Empty specified values compare as exact actual values;
        // absence never becomes a reflected empty name. Local failure remains.
        let uniqueness = if let Some(raw) = raw {
            let mut frames = Vec::new();
            let result = self.form_name_uniqueness(subject, raw.value(), &mut frames);
            self.release_scratch(frames.capacity());
            result?
        } else {
            HtmlFormNameUniqueness::NoSpecifiedName
        };
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(FormNameData { local, uniqueness });
        Ok(HtmlSemanticValue::Known(HtmlFormNameAuthorReport { data }))
    }

    fn form_name_uniqueness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        name: &DocumentString,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlFormNameUniqueness, HtmlSemanticFailure> {
        // Collection membership requires an actual Document ancestor. A known
        // equal form in an unresolved component is insufficient: that component
        // may be detached, in which case the source imposes no collection rule.
        let root = match self.ordinary_tree_root(subject)? {
            Ok(root) => root,
            Err(missing) => return Ok(HtmlFormNameUniqueness::Missing(missing)),
        };
        let root_node = self.node(&root)?;
        self.charge(1)?;
        if !matches!(root_node.kind(), HtmlCapturedNodeKind::Document(_)) {
            return Ok(HtmlFormNameUniqueness::NoFormsCollection);
        }
        let mut missing = None;
        let mut current = Some(root);
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Form)
                && let Some(other) = self.attribute(element, "name")?
            {
                self.charge(1)?;
                self.interpreter
                    .admit_attribute_raw(Some(other.value()))
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                if self.control_equal_units(name.code_units(), other.value().code_units())? {
                    self.charge(1)?;
                    return Ok(HtmlFormNameUniqueness::Duplicate(id));
                }
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
            HtmlFormNameUniqueness::Unique,
            HtmlFormNameUniqueness::Missing,
        ))
    }
}
