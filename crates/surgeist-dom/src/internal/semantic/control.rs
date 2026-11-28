//! Rebuild actual control facts from the retained canonical snapshot/map.
use super::*;
use surgeist_html::values::DocumentString;
use surgeist_html_semantics::{
    HtmlControlFactRecords, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlCustomControlClassification, HtmlCustomErrorFact, HtmlElementReferenceFact,
    HtmlElementReferenceFacts, HtmlElementReferenceFactsLimits, HtmlElementReferenceKey,
    HtmlExplicitElementReference, HtmlFormAssociation, HtmlFormAssociationState,
};

/// Project qualified FACE positives and the existing HTML null-definition negatives.
pub(super) fn native_input(
    source: &DomSnapshot,
    entries: &[Entry],
    control: &mut Control<'_>,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let mut classifications = Vec::new();
    for entry in entries {
        control.charge(1)?;
        let Some(element) = source.node(entry.node)?.as_element() else {
            continue;
        };
        let form_associated = if element.form_associated_custom_disabled().is_some() {
            true
        } else if element.common_state().html().is_some()
            && element.custom_element_definition()
                == crate::internal::element::DomElementDefinition::Null
        {
            false
        } else {
            continue;
        };
        control.reserve(&mut classifications, 1)?;
        // Context::new performs one element qualification and one duplicate flag test.
        control.charge(2)?;
        classifications.push(HtmlCustomControlClassification::new(
            entry.subject.clone(),
            form_associated,
        ));
    }
    Ok(HtmlSemanticContextInput::new(classifications))
}

pub(super) fn with_native_state(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    subjects: &super::nodes::Subjects<'_>,
    control: &mut Control<'_>,
    output_units: usize,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let entries = subjects.entries();
    let mut associations = Vec::new();
    let mut inputs = Vec::new();
    let mut textareas = Vec::new();
    let mut outputs = Vec::new();
    let mut options = Vec::new();
    let mut custom_errors = Vec::new();
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        if let Some((state, _)) = node
            .as_element()
            .and_then(crate::internal::element::DomElement::native_control)
        {
            control.reserve(&mut custom_errors, 1)?;
            custom_errors.push(HtmlCustomErrorFact::new(
                entry.subject.clone(),
                !state.message.is_empty(),
            ));
        }
        if let Some(option) = node.as_option() {
            control.reserve(&mut options, 1)?;
            options.push(surgeist_html_semantics::HtmlOptionControlState::new(
                entry.subject.clone(),
                Some(option.state().flags().selectedness()),
                Some(option.state().flags().dirtiness()),
            ));
        }
        let Some(_) = node
            .as_element()
            .and_then(crate::internal::element::DomElement::native_listed_form_association)
            .map(|_| ())
            .or_else(|| node.as_img().map(|_| ()))
            .or_else(|| {
                node.as_element()
                    .and_then(crate::internal::element::DomElement::form_associated_custom_disabled)
                    .map(|_| ())
            })
        else {
            continue;
        };
        let owner = if let Some(owner) = node.node.control_owner() {
            Some(subjects.subject(control, owner)?)
        } else {
            None
        };
        control.reserve(&mut associations, 1)?;
        associations.push(HtmlFormAssociation::new(
            entry.subject.clone(),
            HtmlFormAssociationState::EstablishedCurrent(owner),
        ));
        if let Some(input) = node.as_input() {
            let state = input.state();
            let flags = state.flags();
            control.reserve(&mut inputs, 1)?;
            let mut observed = surgeist_html_semantics::HtmlInputControlState::new(
                entry.subject.clone(),
                Some(flags.checkedness()),
                Some(flags.dirty_checkedness()),
                Some(flags.dirty_value()),
                Some(flags.indeterminate()),
            );
            if let Some(files) = state.selected_files() {
                control.charge(1)?;
                observed = observed.with_file_selection_empty(files.is_empty());
            }
            inputs.push(observed);
        }
        if let Some(output) = node.as_output() {
            control.reserve(&mut outputs, 1)?;
            outputs.push(surgeist_html_semantics::HtmlOutputDefaultFact::new(
                entry.subject.clone(),
                match output.state().default_value_override() {
                    None => surgeist_html_semantics::HtmlOutputDefaultValueOverride::Null,
                    Some(value) => surgeist_html_semantics::HtmlOutputDefaultValueOverride::Value(
                        value.clone(),
                    ),
                },
            ));
        }
        if let Some(textarea) = node.as_textarea() {
            control.reserve(&mut textareas, 1)?;
            textareas.push(surgeist_html_semantics::HtmlTextareaControlState::new(
                entry.subject.clone(),
                Some(textarea.state().flags().dirty_value()),
            ));
        }
    }
    // Adapter loops use their own work budget. The caller's same finite work
    // bound independently caps the producer's validation AND remaining context
    // join, whose consumed work is retained inside the moved facts. Neither
    // path can borrow an unlimited allowance or clone checked move-only facts.
    let facts = HtmlControlFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlControlFactRecords {
            associations,
            inputs,
            options,
            textareas,
            custom_errors,
        },
        HtmlControlFactsLimits::new(
            control.limits.subjects.saturating_mul(3),
            control.limits.work,
        ),
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
        if let Some(targets) = node.node.reflected_targets() {
            for (slot, key) in [
                (
                    crate::internal::reflected::ReflectedTarget::CommandFor,
                    HtmlElementReferenceKey::CommandFor,
                ),
                (
                    crate::internal::reflected::ReflectedTarget::PopoverTarget,
                    HtmlElementReferenceKey::PopoverTarget,
                ),
            ] {
                let Some(target) = targets.get(slot) else {
                    continue;
                };
                control.charge(1)?;
                let state = match target {
                    None => HtmlExplicitElementReference::Null,
                    Some(target) => {
                        HtmlExplicitElementReference::Element(subjects.subject(control, target)?)
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
    let outputs = surgeist_html_semantics::HtmlOutputDefaultFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        outputs,
        surgeist_html_semantics::HtmlOutputDefaultLimits::new(
            control.limits.subjects,
            output_units,
            control.limits.work,
        ),
    )
    .map_err(DomHtmlCaptureError::OutputDefaults)?;
    control.charge(1)?;
    Ok(input
        .with_output_defaults(outputs)
        .with_controls(facts)
        .with_element_references(references))
}

/// Actual applicable Input values and Textarea API-normalized current values.
pub(super) fn current_values(
    source: &DomSnapshot,
    entries: &[Entry],
    control: &mut Control<'_>,
    unit_limit: usize,
) -> Result<Vec<surgeist_html_semantics::HtmlControlCurrentValue>, DomHtmlCaptureError> {
    let mut values = Vec::new();
    let mut units = 0usize;
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        let (raw, normalize) = if let Some(input) = node.as_input() {
            if !input
                .state()
                .input_type()
                .uses_current_value_for_directionality()
            {
                continue;
            }
            (input.value(), false)
        } else if let Some(textarea) = node.as_textarea() {
            (textarea.state().raw_value().code_units(), true)
        } else {
            continue;
        };
        let mut length = raw.len();
        if normalize {
            length = 0;
            let mut normalizer = surgeist_html::values::HtmlNewlineNormalizer::new();
            for &unit in raw {
                control.charge(1)?;
                if normalizer.push(unit).is_some() {
                    length = length
                        .checked_add(1)
                        .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
                }
            }
        }
        units = units
            .checked_add(length)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if units > unit_limit {
            return Err(DomHtmlCaptureError::Language(
                surgeist_html_semantics::HtmlLanguageDirectionError::CodeUnits {
                    limit: unit_limit,
                    required: units,
                },
            ));
        }
        length
            .checked_mul(std::mem::size_of::<u16>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        control.reserve(&mut values, 1)?;
        let mut copy = Vec::new();
        copy.try_reserve_exact(length)
            .map_err(|_| DomHtmlCaptureError::AllocationFailed)?;
        let mut normalizer = surgeist_html::values::HtmlNewlineNormalizer::new();
        for &unit in raw {
            control.charge(1)?;
            if normalize {
                if let Some(unit) = normalizer.push(unit) {
                    copy.push(unit);
                }
            } else {
                copy.push(unit);
            }
        }
        values.push(surgeist_html_semantics::HtmlControlCurrentValue::new(
            entry.subject.clone(),
            Some(DocumentString::from_utf16(copy)),
        ));
    }
    Ok(values)
}
