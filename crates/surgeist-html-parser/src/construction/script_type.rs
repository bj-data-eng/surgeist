//! Script element type and language attribute classification.

use super::*;
use surgeist_html::{
    construction::HtmlElement, elements::script::HtmlScriptKind, values::DocumentString,
};

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
    work: surgeist_html::bounded::script::HtmlScriptTypeWork,
) -> Result<(), SessionError> {
    control.charge_change(surgeist_html::accounting::script_type_work(work))?;
    Ok(())
}

pub(super) fn classify_script(
    control: &mut ParseControl,
    element: &HtmlElement,
) -> Result<Option<HtmlScriptKind>, SessionError> {
    let explicit = script_attribute(control, element, "type")?;
    let language = script_attribute(control, element, "language")?;
    let mut cursor =
        surgeist_html::bounded::script::HtmlScriptTypeCursor::preparation(explicit, language);
    while let Some(work) = cursor.required_work() {
        script_type_charge(control, work)?;
        cursor.advance();
    }
    // Classification returns the shared kind; the saved metadata slot remains
    // independent and is installed only by the actual preparation algorithm.
    cursor.result().ok_or_else(|| latch_registry_row(control))
}
