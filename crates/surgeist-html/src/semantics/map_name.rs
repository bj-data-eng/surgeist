//! Independent captured map-name author observations.
//! [HTML 877d map name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-map-name).
//! No overall conformance bit, live collection, map association or geometry.

use super::*;
use crate::{HtmlAttributeReport, HtmlSpaceToken};

/// Exact specified name/id relationship, independent of each local grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMapNameIdRelation {
    NoName,
    IdAbsent,
    Equal,
    Different,
}
/// Same ordinary-tree specified-name comparison. Invalid names still participate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMapNameUniqueness {
    NoName,
    Unique,
    Duplicate(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}
/// Delivered independent author observations; inner Missing is not conformance.
/// The private vector contains exactly one admitted record, never a partial set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMapNameAuthorReport {
    data: Vec<MapNameData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct MapNameData {
    local: HtmlAttributeReport<HtmlSpaceToken>,
    id: Option<Attribute>,
    id_relation: HtmlMapNameIdRelation,
    uniqueness: HtmlMapNameUniqueness,
}
impl HtmlMapNameAuthorReport {
    pub fn local(&self) -> &HtmlAttributeReport<HtmlSpaceToken> {
        &self.data[0].local
    }
    /// Present only when a specified name caused the own id to be consulted.
    pub fn id_attribute(&self) -> Option<&Attribute> {
        self.data[0].id.as_ref()
    }
    pub fn id_relation(&self) -> HtmlMapNameIdRelation {
        self.data[0].id_relation
    }
    pub fn uniqueness(&self) -> &HtmlMapNameUniqueness {
        &self.data[0].uniqueness
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn map_name_author(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMapNameAuthorReport>, HtmlSemanticFailure> {
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
        let raw = self.attribute(element, "name")?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let work = HtmlValueInterpreter::map_name_report_work_bound(element.name(), raw)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)?;
        let local = HtmlValueInterpreter::map_name_report_from_borrowed(element.name(), raw);
        self.poll()?;
        let (id, id_relation, uniqueness) = if let Some(raw) = raw {
            let id = self.attribute(element, "id")?;
            let relation = if let Some(id) = id {
                self.charge(1)?;
                self.interpreter
                    .admit_attribute_raw(Some(id.value()))
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                if self.control_equal_units(raw.value().code_units(), id.value().code_units())? {
                    HtmlMapNameIdRelation::Equal
                } else {
                    HtmlMapNameIdRelation::Different
                }
            } else {
                HtmlMapNameIdRelation::IdAbsent
            };
            let mut frames = Vec::new();
            let uniqueness = self.map_name_uniqueness(subject, raw.value(), &mut frames);
            self.release_scratch(frames.capacity());
            let uniqueness = uniqueness?;
            self.charge(4)?;
            (id.cloned(), relation, uniqueness)
        } else {
            (
                None,
                HtmlMapNameIdRelation::NoName,
                HtmlMapNameUniqueness::NoName,
            )
        };
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(MapNameData {
            local,
            id,
            id_relation,
            uniqueness,
        });
        Ok(HtmlSemanticValue::Known(HtmlMapNameAuthorReport { data }))
    }

    fn map_name_uniqueness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        name: &DocumentString,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlMapNameUniqueness, HtmlSemanticFailure> {
        let (root, mut missing) = match self.ordinary_tree_root(subject)? {
            Ok(root) => (root, None),
            Err(HtmlSemanticMissing::Parent(id)) => {
                (id.clone(), Some(HtmlSemanticMissing::Parent(id)))
            }
            Err(missing) => return Ok(HtmlMapNameUniqueness::Missing(missing)),
        };
        let mut current = Some(root);
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Map)
                && let Some(other) = self.attribute(element, "name")?
            {
                self.charge(1)?;
                self.interpreter
                    .admit_attribute_raw(Some(other.value()))
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                if self.control_equal_units(name.code_units(), other.value().code_units())? {
                    self.charge(1)?;
                    return Ok(HtmlMapNameUniqueness::Duplicate(id));
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
            HtmlMapNameUniqueness::Unique,
            HtmlMapNameUniqueness::Missing,
        ))
    }
}
