//! Immutable map area membership, HTML 877d6146487f. No geometry or activation.

use super::*;

/// Source-ordered HTML area subjects from one qualified immutable capture.
/// This is not a live HTMLCollection or a named lookup/image-map association.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaCollection {
    areas: Vec<HtmlCaptureSubjectId>,
}
impl HtmlAreaCollection {
    pub fn areas(&self) -> &[HtmlCaptureSubjectId] {
        &self.areas
    }
    pub fn len(&self) -> usize {
        self.areas.len()
    }
    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn map_areas(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlAreaCollection>, HtmlSemanticFailure> {
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
        let mut frames = Vec::new();
        let result = self.map_areas_inner(node, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }
    fn map_areas_inner(
        &mut self,
        root: HtmlCapturedNode<'a>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlAreaCollection>, HtmlSemanticFailure> {
        let mut areas = Vec::new();
        let mut current = match self.ordinary_tree_next(root, frames)? {
            Ok(current) => current,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        while let Some(id) = current {
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Area)
            {
                self.reserve_nested_output(&mut areas, 1)?;
                self.charge(1)?;
                areas.push(id);
            }
            // Membership does not prune nested maps, foreign wrappers, areas,
            // or any other ordinary element subtree.
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(current) => current,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlAreaCollection { areas }))
    }
}

/// Independent existential name-definition evidence, not map-author conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlUsemapAuthorDefinition {
    Missing,
    NoLeadingHash,
    NameMatch(HtmlCaptureSubjectId),
    NoNameMatch,
}
/// The first source processing match, independent of the name witness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlUsemapProcessingResult {
    Missing,
    NoHash,
    EmptyTail,
    NoMatchingMap,
    Map {
        subject: HtmlCaptureSubjectId,
        matched_by: HtmlUsemapMatch,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlUsemapMatch {
    Id,
    Name,
    IdAndName,
}
/// The selected map has a matching id but no matching specified name.
/// A different later name witness does not remove this diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlUsemapSelectedTargetDiagnostic {
    IdOnlyMatch,
}
/// Qualified immutable reference evidence, without an overall conformance bit.
///
/// The local form, existential name witness, first processing target and derived
/// selected-target diagnostic are distinct. This does not collect areas, certify
/// map name uniqueness/id equality, or perform geometry, activation or mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlUsemapReferenceResult {
    raw: Option<Attribute>,
    reference: crate::HtmlHashNameReferenceReport,
    author_definition: HtmlUsemapAuthorDefinition,
    processing: HtmlUsemapProcessingResult,
}
impl HtmlUsemapReferenceResult {
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.raw.as_ref()
    }
    pub fn reference(&self) -> &crate::HtmlHashNameReferenceReport {
        &self.reference
    }
    pub fn author_definition(&self) -> &HtmlUsemapAuthorDefinition {
        &self.author_definition
    }
    pub fn processing(&self) -> &HtmlUsemapProcessingResult {
        &self.processing
    }
    /// Derived only from the selected map's actual id/name match.
    /// Null processing, Name and IdAndName selections have no diagnostic.
    pub fn selected_target_diagnostic(&self) -> Option<HtmlUsemapSelectedTargetDiagnostic> {
        matches!(
            self.processing,
            HtmlUsemapProcessingResult::Map {
                matched_by: HtmlUsemapMatch::Id,
                ..
            }
        )
        .then_some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch)
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn usemap_reference(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlUsemapReferenceResult>, HtmlSemanticFailure> {
        use crate::attributes::{HashNameScan, HashNameScanStep};
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Img) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "usemap")?;
        // Metadata/admission precedes every read, including for an early hash.
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let reference = if let Some(raw) = raw {
            self.charge(HashNameScan::BEGIN_WORK)?;
            let mut scan = HashNameScan::new(raw.value());
            loop {
                self.charge(HashNameScan::STEP_WORK)?;
                match scan.step() {
                    HashNameScanStep::Continue(next) => scan = next,
                    HashNameScanStep::Complete(report) => break report,
                }
            }
        } else {
            self.charge(1)?;
            crate::HtmlHashNameReferenceReport::missing()
        };
        let mut frames = Vec::new();
        let result = self.usemap_inner(subject, raw, reference, &mut frames);
        // Only current scratch is released. Cumulative allocated bytes and peak
        // remain recorded, including allocation overshoot and terminal failure.
        self.release_scratch(frames.capacity());
        result
    }
    fn usemap_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        raw: Option<&'a Attribute>,
        reference: crate::HtmlHashNameReferenceReport,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlUsemapReferenceResult>, HtmlSemanticFailure> {
        use crate::{HtmlHashNameAuthorForm as Author, HtmlHashNameProcessing as Processing};
        self.charge(1)?;
        let mut author_definition = match reference.author_form() {
            Author::Missing => HtmlUsemapAuthorDefinition::Missing,
            Author::NoLeadingHash => HtmlUsemapAuthorDefinition::NoLeadingHash,
            Author::LeadingHashName => HtmlUsemapAuthorDefinition::NoNameMatch,
        };
        self.charge(1)?;
        let mut processing = match reference.processing() {
            Processing::Missing => HtmlUsemapProcessingResult::Missing,
            Processing::NoHash => HtmlUsemapProcessingResult::NoHash,
            Processing::EmptyTail => HtmlUsemapProcessingResult::EmptyTail,
            Processing::Name => HtmlUsemapProcessingResult::NoMatchingMap,
        };
        self.charge(1)?;
        let need_name = matches!(author_definition, HtmlUsemapAuthorDefinition::NoNameMatch);
        let need_processing = matches!(processing, HtmlUsemapProcessingResult::NoMatchingMap);
        if need_name || need_processing {
            let root = match self.ordinary_tree_root(subject)? {
                Ok(root) => root,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            let mut current = Some(root);
            let mut deferred_missing = None;
            while let Some(id) = current {
                let node = self.node(&id)?;
                if let HtmlCapturedNodeKind::Element(element) = node.kind()
                    && self.known(element)? == Some(KnownHtmlElement::Map)
                {
                    self.charge(1)?;
                    let need_name =
                        matches!(author_definition, HtmlUsemapAuthorDefinition::NoNameMatch);
                    let need_processing =
                        matches!(processing, HtmlUsemapProcessingResult::NoMatchingMap);
                    // When both phases need a name, their slice is identical:
                    // leading # is the first #. Reuse this one comparison.
                    let suffix = if need_processing {
                        reference.processing_name_units().expect("processing Name")
                    } else {
                        reference.author_name_units().expect("leading hash")
                    };
                    let name_matches = if let Some(name) = self.attribute(element, "name")? {
                        self.charge(1)?;
                        self.interpreter
                            .admit_attribute_raw(Some(name.value()))
                            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                        self.control_equal_units(name.value().code_units(), suffix)?
                    } else {
                        false
                    };
                    if need_name && name_matches {
                        self.charge(1)?;
                        author_definition = HtmlUsemapAuthorDefinition::NameMatch(id.clone());
                    }
                    if need_processing {
                        let id_matches = if let Some(attribute) = self.attribute(element, "id")? {
                            self.charge(1)?;
                            self.interpreter
                                .admit_attribute_raw(Some(attribute.value()))
                                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                            self.control_equal_units(attribute.value().code_units(), suffix)?
                        } else {
                            false
                        };
                        if id_matches || name_matches {
                            self.charge(1)?;
                            processing = HtmlUsemapProcessingResult::Map {
                                subject: id,
                                matched_by: match (id_matches, name_matches) {
                                    (true, true) => HtmlUsemapMatch::IdAndName,
                                    (true, false) => HtmlUsemapMatch::Id,
                                    _ => HtmlUsemapMatch::Name,
                                },
                            };
                        }
                    }
                    self.charge(1)?;
                    if !matches!(author_definition, HtmlUsemapAuthorDefinition::NoNameMatch)
                        && !matches!(processing, HtmlUsemapProcessingResult::NoMatchingMap)
                    {
                        break;
                    }
                }
                current = match self.ordinary_tree_next(node, frames)? {
                    Ok(current) => current,
                    Err(missing) => {
                        // Before the first processing match, unknown descendants
                        // can precede every later candidate. Once that match is
                        // fixed, only an existential name witness remains: known
                        // later siblings can prove it despite this unknown branch.
                        if matches!(processing, HtmlUsemapProcessingResult::NoMatchingMap) {
                            return Ok(HtmlSemanticValue::Missing(missing));
                        }
                        if deferred_missing.is_none() {
                            self.charge(1)?;
                            deferred_missing = Some(missing);
                        }
                        self.ordinary_tree_resume(frames)?
                    }
                };
            }
            if let Some(missing) = deferred_missing {
                self.charge(1)?;
                if matches!(author_definition, HtmlUsemapAuthorDefinition::NoNameMatch) {
                    return Ok(HtmlSemanticValue::Missing(missing));
                }
            }
            // Only complete traversal establishes a remaining negative name fact.
        }
        self.charge(1)?;
        let raw = raw.cloned();
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlUsemapReferenceResult {
            raw,
            reference,
            author_definition,
            processing,
        }))
    }
}
