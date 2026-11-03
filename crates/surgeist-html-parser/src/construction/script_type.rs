//! Script element type and language attribute classification.

use super::*;

pub(super) fn script_attribute<'a>(
    control: &mut ParseControl,
    element: &'a HtmlElement,
    local: &str,
) -> Result<Option<&'a DocumentString>, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    for attribute in element.attributes().as_slice() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        if attribute.name().namespace().uri().is_none()
            && exact_ascii(control, Some(attribute.name().local_name()), local)?
        {
            return Ok(Some(attribute.value()));
        }
    }
    Ok(None)
}

pub(super) fn script_type_charge(
    control: &mut ParseControl,
    work: crate::bounded::HtmlScriptTypeWork,
) -> Result<(), SessionError> {
    match work {
        crate::bounded::HtmlScriptTypeWork::RecordVisit(units) => {
            control.charge(HtmlParseWorkCategory::RecordVisit, units)?
        }
        crate::bounded::HtmlScriptTypeWork::InspectByte(units) => {
            control.charge(HtmlParseWorkCategory::InspectByte, units)?
        }
    }
    Ok(())
}

pub(super) fn classify_script(
    control: &mut ParseControl,
    element: &HtmlElement,
) -> Result<Option<HtmlScriptKind>, SessionError> {
    let explicit = script_attribute(control, element, "type")?;
    let language = script_attribute(control, element, "language")?;
    let mut cursor = crate::bounded::HtmlScriptTypeCursor::preparation(explicit, language);
    while let Some(work) = cursor.required_work() {
        script_type_charge(control, work)?;
        cursor.advance();
    }
    // Classification returns the shared kind; the saved metadata slot remains
    // independent and is installed only by the actual preparation algorithm.
    cursor.result().ok_or_else(|| latch_registry_row(control))
}
