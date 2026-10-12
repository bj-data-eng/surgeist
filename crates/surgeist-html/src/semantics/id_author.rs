//! HTML 877d global id author observations, over the captured ordinary tree.
//! DOM's concept-ID applies to competing elements in every namespace.

use super::*;
use crate::{HtmlAttributeReport, HtmlSpaceToken};

/// Exact DOM-ID uniqueness, independent of the local HTML author grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIdUniqueness {
    /// Absent or empty id supplies no DOM ID; local author evidence stays separate.
    NoId,
    Unique,
    Duplicate(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}

/// Independent local syntax and captured uniqueness observations.
/// Known delivery is not overall conformance; an inner Missing remains unavailable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIdAuthorReport {
    data: Vec<IdAuthorData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct IdAuthorData {
    local: HtmlAttributeReport<HtmlSpaceToken>,
    uniqueness: HtmlIdUniqueness,
}
impl HtmlIdAuthorReport {
    pub fn local(&self) -> &HtmlAttributeReport<HtmlSpaceToken> {
        &self.data[0].local
    }
    pub fn uniqueness(&self) -> &HtmlIdUniqueness {
        &self.data[0].uniqueness
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn id_author(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlIdAuthorReport>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let raw = self.attribute(element, "id")?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let work = HtmlValueInterpreter::id_report_work_bound(element.name(), raw)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)?;
        let local = HtmlValueInterpreter::id_report_from_borrowed(element.name(), raw);
        self.poll()?;
        self.charge(1)?;
        let uniqueness = if let Some(raw) = raw.filter(|a| !a.value().is_empty()) {
            let mut frames = Vec::new();
            let result = self.id_uniqueness(subject, raw.value(), &mut frames);
            self.release_scratch(frames.capacity());
            result?
        } else {
            HtmlIdUniqueness::NoId
        };
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(IdAuthorData { local, uniqueness });
        Ok(HtmlSemanticValue::Known(HtmlIdAuthorReport { data }))
    }

    fn id_uniqueness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        value: &DocumentString,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlIdUniqueness, HtmlSemanticFailure> {
        let (root, mut missing) = match self.ordinary_tree_root(subject)? {
            Ok(root) => (root, None),
            // The observed subtree is still in the subject's tree. A duplicate
            // within it can prove failure; absence cannot prove global uniqueness.
            Err(HtmlSemanticMissing::Parent(id)) => {
                (id.clone(), Some(HtmlSemanticMissing::Parent(id)))
            }
            Err(missing) => return Ok(HtmlIdUniqueness::Missing(missing)),
        };
        let mut current = Some(root);
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(element) = node.kind()
                && let Some(other) = self.attribute(element, "id")?
            {
                self.charge(1)?;
                self.interpreter
                    .admit_attribute_raw(Some(other.value()))
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                // The queried ID is nonempty; the exact comparator also excludes
                // empty competitors without inventing another ID grammar.
                if self.control_equal_units(value.code_units(), other.value().code_units())? {
                    self.charge(1)?;
                    return Ok(HtmlIdUniqueness::Duplicate(id));
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
        Ok(missing.map_or(HtmlIdUniqueness::Unique, HtmlIdUniqueness::Missing))
    }
}
