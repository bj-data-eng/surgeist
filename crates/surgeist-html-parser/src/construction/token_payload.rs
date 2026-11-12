//! Validation and release of lexical token payloads handed to the arena.

use super::*;
use surgeist_html::values::DocumentString;

pub(super) fn release_token_attributes(
    mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    let mut disposal_stop = None;
    loop {
        let attribute = if control.stop().is_some() {
            attributes.cleanup_pop(control)?
        } else {
            match attributes.try_pop(control) {
                Ok(attribute) => attribute,
                Err(stop) => {
                    // A stop first observed by this pop must not drop the only
                    // owner of lexical capacity or of the remaining fields.
                    // All incoming owners were checked before token consumption.
                    disposal_stop = Some(stop);
                    attributes.cleanup_pop(control)?
                }
            }
        };
        let Some(attribute) = attribute else { break };
        attribute.name.release(control)?;
        attribute.value.release(control)?;
    }
    attributes.release(control)?;
    disposal_stop.map_or(Ok(()), |stop| Err(SessionError::Stop(stop)))
}

pub(super) fn literal_string(
    control: &mut ParseControl,
    literal: &'static [u8],
) -> Result<FrozenUtf16, SessionError> {
    let mut buffer = Utf16Buffer::new(control)?;
    let result = (|| {
        for &byte in literal {
            control.charge(HtmlParseWorkCategory::InspectByte, 1)?;
            buffer.push_unit(u16::from(byte), control)?;
        }
        Ok::<_, HtmlParseStop>(())
    })();
    if let Err(stop) = result {
        buffer.release(control)?;
        return Err(stop.into());
    }
    Ok(buffer.freeze(control)?)
}

pub(super) fn check_text_source(
    control: &mut ParseControl,
    text: &Text,
) -> Result<(), SessionError> {
    for origin in text.origins.as_slice() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.require_source_range(&origin.source)?;
    }
    Ok(())
}

/// A value that is a run of UTF-16 code units: a string, or the units of a record that is still
/// being built.
pub(super) trait Units16 {
    fn units16(&self) -> &[u16];
}
impl Units16 for [u16] {
    fn units16(&self) -> &[u16] {
        self
    }
}
impl Units16 for DocumentString {
    fn units16(&self) -> &[u16] {
        self.code_units()
    }
}

pub(super) fn exact_string_equal(
    control: &mut ParseControl,
    left: &(impl Units16 + ?Sized),
    right: &(impl Units16 + ?Sized),
) -> Result<bool, SessionError> {
    let (left, right) = (left.units16(), right.units16());
    control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        control.charge(HtmlParseWorkCategory::InspectByte, 4)?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn check_disposal_text(
    control: &mut ParseControl,
    text: &Text,
) -> Result<(), SessionError> {
    control.require_document_string_for_disposal(text.value.allocation(), text.value.text())?;
    text.origins.require_disposal_owner(control)?;
    cleanup_charge(control, HtmlParseWorkCategory::ProviderCall, 1)?;
    if text.value.text().block().holders() != 1 {
        return Err(SessionError::AliasedPayload);
    }
    Ok(())
}

/// Consumes only a checked, uniquely owned lexical payload. Terminal checks
/// inspect ownership solely for disposal and never resume token processing.
fn check_token_kind(token: &TokenKind, control: &mut ParseControl) -> Result<(), SessionError> {
    match token {
        TokenKind::Text(text) | TokenKind::Comment(text) => check_disposal_text(control, text)?,
        TokenKind::ProcessingInstruction { target, data } => {
            check_disposal_text(control, target)?;
            check_disposal_text(control, data)?;
        }
        TokenKind::Doctype(value) => {
            for field in [&value.name, &value.public_id, &value.system_id] {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                if let Some(text) = field {
                    check_disposal_text(control, text)?;
                }
            }
        }
        TokenKind::Tag(value) => {
            check_disposal_text(control, &value.name)?;
            value.attributes.require_disposal_owner(control)?;
            for attribute in value.attributes.as_slice() {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                check_disposal_text(control, &attribute.name)?;
                check_disposal_text(control, &attribute.value)?;
            }
        }
        TokenKind::Eof => {
            if control.stop().is_none() {
                control.checkpoint()?;
            }
        }
    }
    Ok(())
}

/// Active borrowed admission of every owned lexical field, not a source-ID
/// substitute or permission to resume a terminal execution. EOF owns no fields.
pub(crate) fn validate_token_kind(
    token: &TokenKind,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    control.checkpoint()?;
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    check_token_kind(token, control)
}

pub(crate) fn release_token_kind(
    input: &mut Option<TokenKind>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
    let token = input.as_ref().ok_or(SessionError::Consumed)?;
    check_token_kind(token, control)?;
    if control.stop().is_none() {
        // Admit the finite destructive field inspection before consuming the
        // borrowed owner, so disposal cannot strand fields on a late work stop.
        let fields = match token {
            TokenKind::Text(_) | TokenKind::Comment(_) => 1,
            TokenKind::ProcessingInstruction { .. } | TokenKind::Tag(_) => 2,
            TokenKind::Doctype(_) => 3,
            TokenKind::Eof => 0,
        };
        control.charge(HtmlParseWorkCategory::RecordVisit, fields)?;
        payload_write::<Option<TokenKind>>(control, 2)?;
    }
    let Some(token) = input.take() else {
        return Err(SessionError::Consumed);
    };
    match token {
        TokenKind::Text(text) | TokenKind::Comment(text) => text.release(control)?,
        TokenKind::ProcessingInstruction { target, data } => {
            target.release(control)?;
            data.release(control)?;
        }
        TokenKind::Doctype(value) => {
            // The finite owned fields are being disposed, not parsed. Admission
            // above covers active metadata; after a stop cleanup is uncharged.
            for text in [value.name, value.public_id, value.system_id]
                .into_iter()
                .flatten()
            {
                text.release(control)?;
            }
        }
        TokenKind::Tag(value) => {
            value.name.release(control)?;
            release_token_attributes(value.attributes, control)?;
        }
        TokenKind::Eof => {}
    }
    Ok(())
}
