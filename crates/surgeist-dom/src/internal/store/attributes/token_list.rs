//! Source operations on the ordered set associated with an attribute.

use super::*;
use crate::internal::token_list::{
    DomTokenList, DomTokenListEdit, scan_space_tokens, token_position,
};
use std::ops::ControlFlow;
use surgeist_html::bounded::lexical::is_ascii_whitespace;

impl Draft {
    pub(in crate::internal::store) fn edit_token_list(
        &mut self,
        list: DomTokenList,
        edit: &DomTokenListEdit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        validate(edit, || self.source_step(cancel))?;
        let node = self.resolve(list.element().into())?;
        let attributes = self.element(node)?.attributes();
        let position = self.attribute_position(
            attributes,
            AttributeLookup::NullNamespace(list.attribute_name()),
            cancel,
        )?;
        let raw = position
            .map(|index| attributes.get(index).ok_or(DomError::StaleNode))
            .transpose()?
            .map(|attribute| attribute.value());

        let scratch = source::formation::Formation::new(self, cancel)?;
        let serialized = scratch.as_ref().map(source::formation::Formation::fork);
        let (result, value) = {
            let mut tokens = OrderedTokens {
                values: Vec::new(),
                backing: scratch.as_ref().map(source::formation::Formation::fork),
                cancel,
            };
            let mut step = || self.source_step(cancel);
            if let Some(raw) = raw {
                scan_space_tokens(raw.code_units(), &mut step, |token, step| {
                    tokens.append(token, step)?;
                    Ok(ControlFlow::<()>::Continue(()))
                })?;
            }
            let (update, result) = tokens.edit(edit, &mut step)?;
            let value = (update && (raw.is_some() || !tokens.values.is_empty()))
                .then(|| tokens.serialize(serialized.as_ref(), step))
                .transpose()?;
            (result, value)
        };
        if let Some(result) = result {
            self.register_node_ticket(NodeTicket::TokenListResult(node, result), cancel)?;
        }
        if let Some(value) = value {
            let name =
                source::formation::string(list.attribute_name_str(), scratch.as_ref(), cancel)?;
            let prepared =
                self.prepare_attribute_name(node, Some(&Namespace::none()), &name, cancel)?;
            self.apply_attribute_name_at(node, prepared, position, &value, limits, cancel)?;
        }
        Ok(())
    }
}

fn validate(
    edit: &DomTokenListEdit,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    match edit {
        DomTokenListEdit::Add(tokens) | DomTokenListEdit::Remove(tokens) => {
            for token in tokens {
                nonempty(token, &mut step)?;
                no_whitespace(token, &mut step)?;
            }
        }
        DomTokenListEdit::Toggle { token, .. } => {
            nonempty(token, &mut step)?;
            no_whitespace(token, &mut step)?;
        }
        DomTokenListEdit::Replace { token, replacement } => {
            nonempty(token, &mut step)?;
            nonempty(replacement, &mut step)?;
            no_whitespace(token, &mut step)?;
            no_whitespace(replacement, &mut step)?;
        }
    }
    Ok(())
}

fn nonempty(
    token: &DocumentString,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    step()?;
    if token.is_empty() {
        return Err(DomError::EmptyToken);
    }
    Ok(())
}

fn no_whitespace(
    token: &DocumentString,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    for &unit in token.code_units() {
        step()?;
        if is_ascii_whitespace(unit) {
            return Err(DomError::TokenContainsAsciiWhitespace);
        }
    }
    Ok(())
}

struct OrderedTokens<'a, 'cancel> {
    values: Vec<&'a [u16]>,
    backing: Option<source::formation::Formation>,
    cancel: &'cancel AtomicBool,
}

impl<'a> OrderedTokens<'a, '_> {
    fn append(
        &mut self,
        token: &'a [u16],
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if token_position(&self.values, token, &mut *step)?.is_none() {
            self.push(token, step)?;
        }
        Ok(())
    }

    fn push(
        &mut self,
        token: &'a [u16],
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        source::formation::reserve(&mut self.values, 1, self.backing.as_ref(), self.cancel)?;
        step()?;
        self.values.push(token);
        Ok(())
    }

    fn remove(
        &mut self,
        index: usize,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        for _ in index..self.values.len() {
            step()?;
        }
        self.values.remove(index);
        Ok(())
    }

    fn edit(
        &mut self,
        edit: &'a DomTokenListEdit,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(bool, Option<bool>), DomError> {
        match edit {
            DomTokenListEdit::Add(tokens) => {
                for token in tokens {
                    self.append(token.code_units(), step)?;
                }
                Ok((true, None))
            }
            DomTokenListEdit::Remove(tokens) => {
                for token in tokens {
                    if let Some(index) =
                        token_position(&self.values, token.code_units(), &mut *step)?
                    {
                        self.remove(index, &mut *step)?;
                    }
                }
                Ok((true, None))
            }
            DomTokenListEdit::Toggle { token, force } => {
                let token = token.code_units();
                let position = token_position(&self.values, token, &mut *step)?;
                let present = force.unwrap_or(position.is_none());
                let update = match (position, present) {
                    (Some(index), false) => {
                        self.remove(index, step)?;
                        true
                    }
                    (None, true) => {
                        self.push(token, step)?;
                        true
                    }
                    _ => false,
                };
                Ok((update, Some(present)))
            }
            DomTokenListEdit::Replace { token, replacement } => {
                let Some(index) = token_position(&self.values, token.code_units(), &mut *step)?
                else {
                    return Ok((false, Some(false)));
                };
                let replacement = replacement.code_units();
                let existing = token_position(&self.values, replacement, &mut *step)?;
                // Infra replaces the first occurrence of either token and
                // removes the other occurrence if there is one.
                match existing {
                    Some(existing) if existing < index => self.remove(index, step)?,
                    Some(existing) if existing == index => {}
                    Some(existing) => {
                        self.remove(existing, &mut *step)?;
                        step()?;
                        self.values[index] = replacement;
                    }
                    None => {
                        step()?;
                        self.values[index] = replacement;
                    }
                }
                Ok((true, Some(true)))
            }
        }
    }

    fn serialize(
        &self,
        backing: Option<&source::formation::Formation>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<DocumentString, DomError> {
        let mut length = self.values.len().saturating_sub(1);
        for token in &self.values {
            step()?;
            length = length
                .checked_add(token.len())
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let mut units = Vec::new();
        if let Some(backing) = backing {
            let mut header = source::storage::Payload::default();
            header.empty_string()?;
            backing.retain_payload(header, self.cancel)?;
        }
        source::formation::reserve_utf16(&mut units, length, backing, self.cancel)?;
        for (index, token) in self.values.iter().enumerate() {
            if index > 0 {
                step()?;
                units.push(0x20);
            }
            for &unit in *token {
                step()?;
                units.push(unit);
            }
        }
        Ok(DocumentString::from_utf16(units))
    }
}
