//! Rebuild actual control facts from the retained canonical snapshot/map.
use super::*;
use surgeist_html_semantics::{
    HtmlControlFacts, HtmlControlFactsLimits, HtmlFormAssociation, HtmlFormAssociationState,
};

pub(super) fn with_controls(
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
        let Some(state) = node
            .as_element()
            .and_then(crate::DomElement::fieldset_state)
        else {
            continue;
        };
        let owner = if let Some(owner) = state.form_owner() {
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
    Ok(input.with_controls(facts))
}
