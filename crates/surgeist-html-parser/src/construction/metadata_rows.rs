//! Reads and writes of the per-node metadata rows in the payload registry.

use super::*;
use surgeist_html::values::DocumentString;

pub(super) fn find_textarea_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<Option<(usize, TextareaState)>, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let fields = payloads
        .node_joins
        .get(node)
        .ok_or(SessionError::ForeignNode)?;
    let Some(slot) = fields.textarea() else {
        return Ok(None);
    };
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::Textarea {
        node: Some(owner),
        state,
    }) = payloads.values.get(slot)
    else {
        return Err(SessionError::WrongToken);
    };
    if *owner != node {
        return Err(SessionError::WrongToken);
    }
    control.charge(
        HtmlParseWorkCategory::CopyByte,
        mem::size_of::<TextareaState>() as u64,
    )?;
    Ok(Some((slot, *state)))
}

pub(super) fn admitted_input_string<'a>(
    payloads: &'a PayloadStorage,
    c: &mut ParseControl,
    slot: usize,
) -> Result<&'a DocumentString, SessionError> {
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value, allocation, ..
    }) = payloads.values.get(slot)
    else {
        return Err(SessionError::InputMetadataUnavailable);
    };
    c.require_document_string(allocation, value)?;
    Ok(value)
}

pub(super) fn textarea_raw<'a>(
    payloads: &'a PayloadStorage,
    control: &mut ParseControl,
    raw: usize,
) -> Result<&'a DocumentString, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value,
        allocation,
        location: FieldRef::Detached,
        ..
    }) = payloads.values.get(raw)
    else {
        return Err(SessionError::TextareaMetadataUnavailable);
    };
    control.require_document_string(allocation, value)?;
    Ok(value)
}

pub(super) fn find_form_association_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<Option<(usize, FormAssociationState)>, SessionError> {
    if let Some(slot) = payloads.joined(control, node, Join::FormAssociation)? {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(RetainedPayload::FormAssociation { node: owner, state }) =
            payloads.values.get(slot)
            && *owner == Some(node)
        {
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<FormAssociationState>() as u64,
            )?;
            return Ok(Some((slot, *state)));
        }
    }
    Ok(None)
}

pub(super) fn set_form_association_state(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    slot: usize,
    state: FormAssociationState,
) -> Result<(), SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    payload_write::<FormAssociationState>(control, 1)?;
    let Some(RetainedPayload::FormAssociation { state: current, .. }) =
        storage.payloads.values.get_mut(slot)
    else {
        return Err(SessionError::ImageFormMetadataUnavailable);
    };
    *current = state;
    Ok(())
}

pub(super) fn script_metadata(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<(usize, ScriptMetadata), SessionError> {
    if let Some(position) = payloads.joined(control, node, Join::Script)? {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(RetainedPayload::Script {
            node: owner,
            metadata,
        }) = payloads.values.get(position)
            && *owner == Some(node)
        {
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<ScriptMetadata>() as u64,
            )?;
            metadata.admit_clone(control)?;
            return Ok((position, metadata.clone()));
        }
    }
    Err(SessionError::ScriptMetadataUnavailable)
}

pub(super) fn set_script_metadata(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    slot: usize,
    value: &ScriptMetadata,
) -> Result<(), SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    payload_write::<ScriptMetadata>(control, 1)?;
    let RetainedPayload::Script { metadata, .. } = storage
        .payloads
        .values
        .get_mut(slot)
        .ok_or(SessionError::ScriptMetadataUnavailable)?
    else {
        return Err(SessionError::ScriptMetadataUnavailable);
    };
    value.admit_clone(control)?;
    *metadata = value.clone();
    Ok(())
}
