//! Rebuild actual control facts from the retained canonical snapshot/map.
use super::*;
use surgeist_html_semantics::{
    HtmlControlFacts, HtmlControlFactsLimits, HtmlElementReferenceFact, HtmlElementReferenceFacts,
    HtmlElementReferenceFactsLimits, HtmlElementReferenceKey, HtmlExplicitElementReference,
    HtmlFormAssociation, HtmlFormAssociationState,
};

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
        let Some((state, _)) = node
            .as_element()
            .and_then(crate::DomElement::native_control)
        else {
            continue;
        };
        let owner = if let Some(owner) = state.owner {
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
