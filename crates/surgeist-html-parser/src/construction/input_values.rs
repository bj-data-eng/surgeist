//! Input type classification and the copies of an input value between owners.

use super::*;
#[cfg(test)]
use surgeist_html::construction::HtmlNodeKind;
use surgeist_html::values::DocumentString;

pub(crate) fn input_tag_state(
    control: &mut ParseControl,
    tag: &Tag,
) -> Result<surgeist_html::attributes::HtmlInputType, SessionError> {
    control.require_document_string(tag.name.value.allocation(), tag.name.value.text())?;
    tag.attributes.require_disposal_owner(control)?;
    for attribute in tag.attributes.as_slice() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if exact_ascii(control, Some(attribute.name.value.text()), "type")? {
            let value = attribute.value.value.text();
            control.require_document_string(attribute.value.value.allocation(), value)?;
            return input_type_state(control, value);
        }
    }
    Ok(surgeist_html::attributes::HtmlInputType::missing_value_default())
}

pub(super) fn input_type_state(
    control: &mut ParseControl,
    value: &DocumentString,
) -> Result<surgeist_html::attributes::HtmlInputType, SessionError> {
    let mut cursor = surgeist_html::elements::input::HtmlInputTypeCursor::new(Some(value));
    while let Some(work) = cursor.required_work() {
        control.charge_change(surgeist_html::accounting::input_rule_work(work))?;
        cursor.advance();
    }
    // The loop above admitted every recognition step, so the result exists.
    cursor.result().ok_or_else(|| latch_registry_row(control))
}

pub(super) fn validate_initial_input_origins(
    control: &mut ParseControl,
    tag: &Tag,
) -> Result<(), SessionError> {
    let mut policy = surgeist_html::elements::input::HtmlInputValueSanitization::String(
        surgeist_html::elements::input::HtmlInputSanitization::StripCrLf,
    );
    for attribute in tag.attributes.as_slice() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if exact_ascii(control, Some(attribute.name.value.text()), "type")? {
            policy = input_type_state(control, attribute.value.value.text())?.sanitization();
        }
        if !exact_ascii(control, Some(attribute.name.value.text()), "value")? {
            continue;
        }
        let text = &attribute.value;
        let mut covered = 0;
        for origin in text.origins.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.require_source_range(&origin.source)?;
            if origin.start != covered
                || origin.end <= origin.start
                || origin.end > text.value.text().len()
            {
                return Err(SessionError::OriginSplitUnavailable);
            }
            if let surgeist_html::elements::input::HtmlInputValueSanitization::String(policy) =
                policy
                && policy != surgeist_html::elements::input::HtmlInputSanitization::Preserve
            {
                let Some(units) = text.value.text().code_units().get(origin.start..origin.end)
                else {
                    return Err(latch_registry_row(control));
                };
                let mut omitted = 0usize;
                for &unit in units {
                    control.charge_unit_inspection()?;
                    if !policy.retain_unit(unit) {
                        // At most one per unit of the range being counted.
                        omitted = omitted.saturating_add(1);
                    }
                }
                if omitted != 0 && omitted != units.len() {
                    return Err(SessionError::OriginSplitUnavailable);
                }
            }
            covered = origin.end;
        }
        if covered != text.value.text().len() {
            return Err(SessionError::OriginSplitUnavailable);
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn find_input_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<(usize, InputState), SessionError> {
    for payload in &payloads.values {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::Input {
            node: Some(owner),
            raw,
            complete: true,
            state,
            ..
        } = payload
            && *owner == node
        {
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                (mem::size_of::<usize>() + mem::size_of::<InputState>()) as u64,
            )?;
            return Ok((*raw, *state));
        }
    }
    Err(SessionError::InputMetadataUnavailable)
}

#[cfg(test)]
pub(super) fn find_input_sources<'a>(
    payloads: &'a PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError> {
    let (_, state) = find_input_state(payloads, control, node)?;
    if !state.value_rewritten_during_creation {
        return Ok(None);
    }
    for payload in &payloads.values {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::InputSource {
            node: Some(owner),
            token,
            value_attribute: Some(value),
        } = payload
            && *owner == node
        {
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<(&crate::HtmlSourceRange, &crate::HtmlSourceRange)>() as u64,
            )?;
            return Ok(Some((token, value)));
        }
    }
    Err(SessionError::InputMetadataUnavailable)
}

#[cfg(test)]
pub(super) fn append_input_content(
    records: ConstructionView<'_>,
    control: &mut ParseControl,
    node: usize,
    destination: &mut Utf16Buffer,
    absent_fallback: &[u16],
) -> Result<(), SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let HtmlNodeKind::Element(element) = records
        .node_at(node)
        .ok_or(SessionError::ForeignNode)?
        .kind()
    else {
        return Err(SessionError::InputMetadataUnavailable);
    };
    for attribute in element.attributes().as_slice() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if attribute.name().namespace().uri().is_none()
            && exact_ascii(control, Some(attribute.name().local_name()), "value")?
        {
            destination.extend_units(attribute.value().code_units(), control)?;
            return Ok(());
        }
    }
    if !absent_fallback.is_empty() {
        destination.extend_units(absent_fallback, control)?;
    }
    Ok(())
}

/// Copy actual surviving origin groups. A mixed aggregate cannot be split by
/// guessing source offsets; per-codepoint tokenizer origins establish the cut.
pub(super) fn copy_input_value(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    control: &mut ParseControl,
    slot: usize,
    policy: surgeist_html::elements::input::HtmlInputSanitization,
) -> Result<FieldCopy, SessionError> {
    Ok(copy_input_value_core(
        payloads,
        evidence,
        control,
        slot,
        policy,
        None,
        surgeist_html::elements::input::HtmlInputEmailMode::Single,
    )?
    .0)
}

/// Preserve a current/default value from its actual group. The complete
/// lexical path keeps its original charges; only incomplete lexical coverage
/// reaches the existing owner that requires qualified input-computation proof.
pub(super) fn copy_preserved_input_value(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    control: &mut ParseControl,
    slot: usize,
) -> Result<FieldCopy, SessionError> {
    Ok(copy_input_value_core(
        payloads,
        evidence,
        control,
        slot,
        surgeist_html::elements::input::HtmlInputSanitization::Preserve,
        None,
        surgeist_html::elements::input::HtmlInputEmailMode::Single,
    )?
    .0)
}

pub(super) fn copy_selected_input_value(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    control: &mut ParseControl,
    slot: usize,
    policy: surgeist_html::elements::input::HtmlInputSanitization,
    mode: surgeist_html::elements::input::HtmlInputEmailMode,
) -> Result<(FieldCopy, Option<usize>), SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value, allocation, ..
    }) = payloads.values.get(slot)
    else {
        return Err(SessionError::InputMetadataUnavailable);
    };
    control.require_document_string(allocation, value)?;
    payload_write::<surgeist_html::elements::input::HtmlInputSanitizationCursor<'_>>(control, 1)?;
    let cursor = surgeist_html::elements::input::HtmlInputSanitizationCursor::new(
        policy,
        value.code_units(),
        mode,
    );
    copy_input_value_core(
        payloads,
        evidence,
        control,
        slot,
        policy,
        Some(cursor),
        mode,
    )
}

/// Whole retention or omission of a real generated field. This is reached
/// only after incomplete lexical coverage, then requires actual computation
/// lineage; lexical, storage and terminal errors do not select this path.
fn copy_whole_generated_input_value(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    c: &mut ParseControl,
    source: usize,
    policy: surgeist_html::elements::input::HtmlInputSanitization,
    mode: surgeist_html::elements::input::HtmlInputEmailMode,
) -> Result<(FieldCopy, Option<usize>), SessionError> {
    let value = admitted_input_string(payloads, c, source)?;
    payload_write::<surgeist_html::elements::input::HtmlInputSanitizationCursor<'_>>(c, 1)?;
    let mut cursor = surgeist_html::elements::input::HtmlInputSanitizationCursor::new(
        policy,
        value.code_units(),
        mode,
    );
    let mut range = next_input_range(c, &mut cursor)?;
    let mut retained = 0usize;
    for (position, &unit) in value.code_units().iter().enumerate() {
        while range.is_some_and(|range| range.end() <= position) {
            payload_write::<Option<surgeist_html::elements::input::HtmlInputRetainedRange>>(c, 1)?;
            range = next_input_range(c, &mut cursor)?;
        }
        c.charge_unit_inspection()?;
        if range.is_some_and(|range| range.start() <= position && position < range.end())
            && cursor.retain_unit(unit)
        {
            if cursor.replacement_unit(position).is_some() {
                return Err(SessionError::OriginSplitUnavailable);
            }
            payload_write::<usize>(c, 1)?;
            // At most one per unit of the value being scanned.
            retained = retained.saturating_add(1);
        }
    }
    while cursor.required_work().is_some() {
        next_input_range(c, &mut cursor)?;
    }
    let count = cursor.email_value_count();
    if retained == value.len() {
        return provenance::copy_generated_input_value(payloads, evidence, c, source)
            .map(|field| (field, count));
    }
    if retained != 0 {
        return Err(SessionError::OriginSplitUnavailable);
    }
    let Some(RetainedPayload::String {
        origins: Some(group),
        ..
    }) = payloads.values.get(source)
    else {
        return Err(latch_registry_row(c));
    };
    let evidence = evidence.ok_or(SessionError::WrongToken)?;
    evidence.qualify_generated_copy(c, payloads, group, value.code_units())?;
    let original = evidence.copy_group_reference(c, group)?;
    Ok((
        FieldCopy {
            value: literal_string(c, b"")?,
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
            original: Some(original),
        },
        count,
    ))
}

fn next_input_range(
    control: &mut ParseControl,
    cursor: &mut surgeist_html::elements::input::HtmlInputSanitizationCursor<'_>,
) -> Result<Option<surgeist_html::elements::input::HtmlInputRetainedRange>, SessionError> {
    while let Some(work) = cursor.required_work() {
        control.charge_change(surgeist_html::accounting::input_rule_work(work))?;
        payload_write::<surgeist_html::elements::input::HtmlInputSanitizationCursor<'_>>(
            control, 1,
        )?;
        payload_write::<Option<surgeist_html::elements::input::HtmlInputRetainedRange>>(
            control, 1,
        )?;
        if let Some(range) = cursor.advance() {
            return Ok(Some(range));
        }
    }
    Ok(None)
}

fn copy_input_value_core(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    control: &mut ParseControl,
    slot: usize,
    policy: surgeist_html::elements::input::HtmlInputSanitization,
    mut selection: Option<surgeist_html::elements::input::HtmlInputSanitizationCursor<'_>>,
    generated_mode: surgeist_html::elements::input::HtmlInputEmailMode,
) -> Result<(FieldCopy, Option<usize>), SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value,
        origins,
        allocation,
        ..
    }) = payloads.values.get(slot)
    else {
        return Err(SessionError::InputMetadataUnavailable);
    };
    control.require_document_string(allocation, value)?;
    let mut buffer = Utf16Buffer::new(control)?;
    let mut copied = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let result = (|| {
        let mut range = if let Some(cursor) = &mut selection {
            next_input_range(control, cursor)?
        } else {
            None
        };
        let mut covered = 0;
        if let Some(origins) = origins {
            origins.require_disposal_owner(control)?;
            let evidence = evidence.ok_or(SessionError::WrongToken)?;
            let mut cursor = provenance::GroupOriginCursor::new(evidence, origins, control)?;
            let read = (|| {
                while let Some(origin) = cursor.next(evidence, control)? {
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    control.require_source_range(&origin.source)?;
                    if origin.start != covered
                        || origin.end <= origin.start
                        || origin.end > value.len()
                    {
                        return Err(SessionError::OriginSplitUnavailable);
                    }
                    let Some(origin_units) = value.code_units().get(origin.start..origin.end)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    let mut omitted = 0usize;
                    for (position, &unit) in (origin.start..origin.end).zip(origin_units) {
                        control.charge_unit_inspection()?;
                        let keep = if let Some(cursor) = &mut selection {
                            while range.is_some_and(|range| range.end() <= position) {
                                payload_write::<
                                    Option<surgeist_html::elements::input::HtmlInputRetainedRange>,
                                >(control, 1)?;
                                range = next_input_range(control, cursor)?;
                            }
                            range.is_some_and(|range| {
                                range.start() <= position && position < range.end()
                            }) && cursor.retain_unit(unit)
                        } else {
                            policy.retain_unit(unit)
                        };
                        if !keep {
                            // At most one per unit of the origin being scanned.
                            omitted = omitted.saturating_add(1);
                        }
                    }
                    if omitted != 0 && omitted != origin_units.len() {
                        return Err(SessionError::OriginSplitUnavailable);
                    }
                    if omitted == 0 {
                        let start = buffer.len();
                        let mut transformed = false;
                        if matches!(
                            policy,
                            surgeist_html::elements::input::HtmlInputSanitization::Temporal(_)
                        ) {
                            for (position, &unit) in (origin.start..origin.end).zip(origin_units) {
                                control.charge_unit_inspection()?;
                                let replacement = selection
                                    .as_ref()
                                    .and_then(|cursor| cursor.replacement_unit(position));
                                transformed |= replacement.is_some();
                                buffer.extend_units(&[replacement.unwrap_or(unit)], control)?;
                            }
                        } else {
                            buffer.extend_units(origin_units, control)?;
                        }
                        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                        payload_write::<HtmlParseTextOrigin>(control, 1)?;
                        copied.try_push_geometric(
                            control,
                            HtmlParseTextOrigin {
                                start,
                                end: buffer.len(),
                                source: origin.source.clone(),
                                kind: if transformed { crate::parser::tokenizer::token::HtmlParseOriginKind::InputValueNormalization } else { origin.kind },
                            },
                        )?;
                    }
                    covered = origin.end;
                }
                Ok::<_, SessionError>(())
            })();
            cursor.release(control)?;
            read?;
        }
        if covered != value.len() {
            if origins.is_none() {
                return Err(SessionError::OriginSplitUnavailable);
            }
            return Ok(true);
        }
        if let Some(cursor) = &mut selection {
            while cursor.required_work().is_some() {
                next_input_range(control, cursor)?;
            }
        }
        payload_write::<Text>(control, 1)?;
        Ok::<_, SessionError>(false)
    })();
    let incomplete = match result {
        Ok(incomplete) => incomplete,
        Err(error) => {
            buffer.release(control)?;
            copied.release(control)?;
            return Err(error);
        }
    };
    if incomplete {
        buffer.release(control)?;
        copied.release(control)?;
        return copy_whole_generated_input_value(
            payloads,
            evidence,
            control,
            slot,
            policy,
            generated_mode,
        );
    }
    let original = match origins
        .as_ref()
        .map(|group| {
            evidence
                .ok_or(SessionError::WrongToken)?
                .copy_group_reference(control, group)
        })
        .transpose()
    {
        Ok(original) => original,
        Err(error) => {
            buffer.release(control)?;
            copied.release(control)?;
            return Err(error);
        }
    };
    match buffer.freeze(control) {
        Ok(value) => Ok((
            FieldCopy {
                value,
                origins: copied,
                original,
            },
            selection
                .as_ref()
                .and_then(|cursor| cursor.email_value_count()),
        )),
        Err(stop) => {
            copied.release(control)?;
            Err(stop.into())
        }
    }
}
