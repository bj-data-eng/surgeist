//! Rebuild actual control facts from the retained canonical snapshot/map.
use super::*;
use surgeist_html_semantics::{
    HtmlControlFacts, HtmlControlFactsLimits, HtmlCustomControlClassification,
    HtmlElementReferenceFact, HtmlElementReferenceFacts, HtmlElementReferenceFactsLimits,
    HtmlElementReferenceKey, HtmlExplicitElementReference, HtmlFormAssociation,
    HtmlFormAssociationState,
};

/// Establish only actual HTML/Null-definition negatives. Neither a valid name
/// nor an incomplete registry proves this classification.
pub(super) fn native_input(
    source: &DomSnapshot,
    entries: &[Entry],
    control: &mut Control<'_>,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let mut classifications = Vec::new();
    for entry in entries {
        control.charge(1)?;
        if let Some(element) = source.node(entry.node)?.as_element()
            && element.common_state().html().is_some()
            && element.custom_element_definition() == crate::DomElementDefinition::Null
        {
            control.reserve(&mut classifications, 1)?;
            // Context::new performs one element qualification and i previous
            // duplicate comparisons for this record; admit that exact envelope
            // before entering its non-cancelable bounded validation loop.
            control.charge(
                classifications
                    .len()
                    .checked_add(1)
                    .ok_or(DomHtmlCaptureError::RepresentationExhausted)?,
            )?;
            classifications.push(HtmlCustomControlClassification::new(
                entry.subject.clone(),
                false,
            ));
        }
    }
    Ok(HtmlSemanticContextInput::new(classifications))
}

pub(super) fn with_native_state(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    entries: &[Entry],
    control: &mut Control<'_>,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let mut associations = Vec::new();
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        let Some(_) = node
            .as_element()
            .and_then(crate::DomElement::native_control)
        else {
            continue;
        };
        let owner = if let Some(owner) = node.node.control_owner {
            let mut mapped = None;
            for candidate in entries {
                control.charge(1)?;
                if candidate.node == owner {
                    mapped = Some(candidate.subject.clone());
                    break;
                }
            }
            Some(mapped.ok_or(DomHtmlCaptureError::UnmappedSubject)?)
        } else {
            None
        };
        control.reserve(&mut associations, 1)?;
        associations.push(HtmlFormAssociation::new(
            entry.subject.clone(),
            HtmlFormAssociationState::EstablishedCurrent(owner),
        ));
    }
    // Adapter loops use their own work budget. The caller's same finite work
    // bound independently caps the producer's validation AND remaining context
    // join, whose consumed work is retained inside the moved facts. Neither
    // path can borrow an unlimited allowance or clone checked move-only facts.
    let facts = HtmlControlFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        associations,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        HtmlControlFactsLimits::new(control.limits.subjects, control.limits.work),
    )
    .map_err(DomHtmlCaptureError::Controls)?;
    control.charge(1)?;
    let extent = control
        .limits
        .subjects
        .checked_mul(2)
        .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
    let mut references = Vec::new();
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        if let Some(button) = node.as_button() {
            for (key, target) in [
                (
                    HtmlElementReferenceKey::CommandFor,
                    button.explicit_command_for(),
                ),
                (
                    HtmlElementReferenceKey::PopoverTarget,
                    button.explicit_popover_target(),
                ),
            ] {
                control.charge(1)?;
                let state = match target {
                    None => HtmlExplicitElementReference::Null,
                    Some(target) => {
                        let mut found = None;
                        for candidate in entries {
                            control.charge(1)?;
                            if candidate.node == target {
                                found = Some(candidate.subject.clone());
                                break;
                            }
                        }
                        HtmlExplicitElementReference::Element(
                            found.ok_or(DomHtmlCaptureError::UnmappedSubject)?,
                        )
                    }
                };
                control.reserve_extent(&mut references, 1, extent)?;
                references.push(HtmlElementReferenceFact::new(
                    entry.subject.clone(),
                    key,
                    state,
                ));
            }
        }
    }
    let references = HtmlElementReferenceFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        references,
        HtmlElementReferenceFactsLimits::new(extent, control.limits.work),
    )
    .map_err(DomHtmlCaptureError::ElementReferences)?;
    control.charge(1)?;
    Ok(input
        .with_controls(facts)
        .with_element_references(references))
}
