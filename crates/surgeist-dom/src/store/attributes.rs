use super::*;
use crate::element::{ElementFormation, FormedElement, NonceUpdate};
use crate::names::{attribute_name, name_units, valid_attribute_local_name};
use crate::{DomAttribute as Attribute, DomQualifiedName as QualifiedName};
use surgeist_html::values::Namespace;

/// Actual native edit versus the Input Color cloning step whose current value
/// will be copied by source overlay, without resolving its default again.
#[derive(Clone, Copy)]
pub(super) enum AttributeCause {
    Change,
    CloneInputColor,
}
#[derive(Clone, Copy)]
pub(super) enum AttributeChange {
    Added,
    Set,
    Removed,
}

// The validated name and its admission remain one argument across every
// fallible candidate step. Its backing drops before its formation on failure.
struct AttributeWrite<'a> {
    node: Key,
    position: Option<usize>,
    name: QualifiedName,
    value: &'a DocumentString,
    cause: AttributeCause,
    name_formation: Option<source::formation::Formation>,
}
#[derive(Clone)]
pub(super) struct AttributeSuffix {
    subject: Key,
    changed: QualifiedName,
    old_value: Option<DocumentString>,
    new_value: Option<DocumentString>,
    cause: AttributeCause,
}
impl AttributeSuffix {
    /// The old and new values of the attribute that the effect retains.
    pub(super) fn values(&self) -> impl Iterator<Item = &DocumentString> {
        [self.old_value.as_ref(), self.new_value.as_ref()]
            .into_iter()
            .flatten()
    }
    pub(super) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        let mut payload = source::storage::Payload::default();
        step()?;
        payload.name(&self.changed)?;
        for value in [self.old_value.as_ref(), self.new_value.as_ref()]
            .into_iter()
            .flatten()
        {
            step()?;
            payload.listed_string(value)?;
        }
        Ok(payload)
    }
}

fn attribute_units(attribute: &Attribute) -> Result<usize, DomError> {
    name_units(attribute.name())
        .and_then(|units| units.checked_add(attribute.value().len()))
        .ok_or(DomError::RepresentationExhausted)
}

fn retain_string(
    value: &DocumentString,
    budget: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    if let Some(budget) = budget {
        budget.step(cancel)?;
        let mut payload = source::storage::Payload::default();
        payload.string(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}
/// Retains `value` as an attribute value of the draft's store: the block of a borrowed value is
/// listed once for the transaction and the occurrence is priced by its length, so a parsed value
/// is never charged the capacity of the block it borrows.
fn retain_attribute_value(
    draft: &Draft,
    value: &DocumentString,
    budget: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    draft.list_block(value, cancel)?;
    if let Some(budget) = budget {
        budget.step(cancel)?;
        let mut payload = source::storage::Payload::default();
        payload.listed_string(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}
impl Draft {
    /// Lists the block a retained value borrows, once for the transaction, so that the retention
    /// can price the value by its length. An owner lists nothing.
    pub(in crate::store) fn list_block(
        &self,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.blocks.adopt(
            value,
            |next| self.admit_custody_usage(next, cancel),
            || self.source_step(cancel),
        )
    }
}
fn retain_name(
    value: &QualifiedName,
    budget: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<QualifiedName, DomError> {
    if let Some(budget) = budget {
        budget.step(cancel)?;
        let mut payload = source::storage::Payload::default();
        payload.name(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}

impl Draft {
    pub(super) fn form_detached_attr(
        &mut self,
        document: Key,
        namespace: Option<&Namespace>,
        name: &DocumentString,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Attribute, DomError> {
        let mut normalized = name_registry::Normalized::new(&self.names);
        let raw = if let Some(namespace) = namespace {
            attribute_name(
                namespace,
                name,
                &mut || self.name_step(cancel),
                &mut |input, lower| normalized.units(input, lower, formation, cancel),
                &mut |value| retain_string(value, formation, cancel),
            )?
        } else {
            if !valid_attribute_local_name(name, &mut || self.name_step(cancel))? {
                return Err(DomError::InvalidCharacter);
            }
            let local =
                if self.document_metadata(document)?.class() == crate::DomDocumentClass::Html {
                    crate::names::ascii_lowercase(
                        name,
                        &mut || self.name_step(cancel),
                        &mut |input, lower| normalized.units(input, lower, formation, cancel),
                        &mut |value| retain_string(value, formation, cancel),
                    )?
                } else {
                    retain_string(name, formation, cancel)?
                };
            surgeist_html::values::QualifiedName::new(Namespace::none(), None, local)
        };
        let name = self.bind_name(&raw, formation, cancel)?;
        let value = source::formation::string("", formation, cancel)?;
        Ok(Attribute::from_parts(name, value))
    }

    pub(super) fn set_detached_attr_value(
        &mut self,
        key: Key,
        value: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = self.node(key)?;
        let transaction::KindRead::Attr(old) = node.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        if node.attr_owner().is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        if self.attribute_string_equal(old.value().code_units(), value.code_units(), cancel)? {
            return Ok(());
        }
        let units = self
            .units
            .checked_sub(old.value().len())
            .and_then(|units| units.checked_add(value.len()))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let value = retain_string(value, formation.as_ref(), cancel)?;
        let replacement = Attribute::from_parts(old.name().clone(), value);
        self.replace_attr_value_formed(key, replacement, formation, cancel)?;
        self.units = units;
        Ok(())
    }

    fn attribute_string_equal(
        &self,
        left: &[u16],
        right: &[u16],
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        if left.len() != right.len() {
            return Ok(false);
        }
        for (left, right) in left.iter().zip(right) {
            self.name_step(cancel)?;
            if left != right {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn attribute_namespace_equal(
        &self,
        left: Option<&DocumentString>,
        right: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        match (left, right) {
            (None, None) => Ok(true),
            (Some(left), Some(right)) => {
                self.attribute_string_equal(left.code_units(), right.code_units(), cancel)
            }
            _ => Ok(false),
        }
    }
    fn attribute_qualified_equal(
        &self,
        name: &QualifiedName,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let prefix = name.prefix();
        let offset = prefix.map_or(Ok(0), |p| {
            p.len()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)
        })?;
        if value.len()
            != offset
                .checked_add(name.local_name().len())
                .ok_or(DomError::RepresentationExhausted)?
        {
            return Ok(false);
        }
        if let Some(prefix) = prefix {
            if !self.attribute_string_equal(
                prefix.code_units(),
                &value.code_units()[..prefix.len()],
                cancel,
            )? {
                return Ok(false);
            }
            self.name_step(cancel)?;
            if value.code_units()[prefix.len()] != 0x3a {
                return Ok(false);
            }
        }
        self.attribute_string_equal(
            name.local_name().code_units(),
            &value.code_units()[offset..],
            cancel,
        )
    }
    fn attribute_position(
        &self,
        attributes: transaction::AttributeRead<'_>,
        namespace: Option<&Namespace>,
        name: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<Option<usize>, DomError> {
        for (position, attribute) in attributes.iter().enumerate() {
            self.source_step(cancel)?;
            let matched = if let Some(namespace) = namespace {
                self.attribute_namespace_equal(
                    attribute.name().namespace_uri(),
                    namespace.uri(),
                    cancel,
                )? && self.attribute_string_equal(
                    attribute.name().local_name().code_units(),
                    name.code_units(),
                    cancel,
                )?
            } else {
                self.attribute_qualified_equal(attribute.name(), name, cancel)?
            };
            if matched {
                return Ok(Some(position));
            }
        }
        Ok(None)
    }
    pub(super) fn set_nonce(
        &mut self,
        node: Key,
        value: &DocumentString,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancelled)?;
        let element = self
            .node(node)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .language();
        let old = element.nonce().ok_or(DomError::NonceInapplicable)?.len();
        let units = (self.units - old)
            .checked_add(value.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let candidate = source::formation::Formation::new(self, cancelled)?;
        let value = retain_attribute_value(self, value, candidate.as_ref(), cancelled)?;
        let mut original = source::storage::Payload::default();
        if candidate.is_some() {
            original.listed_string(&value)?;
        }
        let (value, controls) = {
            let mut admit = |next| self.admit_custody_usage(next, cancelled);
            let mut context = ElementFormation::new(
                self.custody_registry(),
                &self.blocks,
                candidate.as_ref(),
                cancelled,
                &mut admit,
            );
            let value = context.nonce(value, original)?;
            (value, context.into_children())
        };
        self.with_element_fields_formed(node, cancelled, candidate, |element| {
            element.set_nonce_payload(value)
        })?;
        // The selected language dependency has now adopted the exact group.
        drop(controls);
        self.units = units;
        Ok(())
    }
    pub(super) fn set_attribute(
        &mut self,
        node: Key,
        namespace: Option<&Namespace>,
        name: &DocumentString,
        value: &DocumentString,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let element = self.element(node)?;
        let mut name_formation = source::formation::Formation::new(self, cancelled)?;
        let mut normalized = name_registry::Normalized::new(&self.names);
        let (position, raw_name) = if let Some(namespace) = namespace {
            let name = attribute_name(
                namespace,
                name,
                &mut || self.name_step(cancelled),
                &mut |input, lower| {
                    normalized.units(input, lower, name_formation.as_ref(), cancelled)
                },
                &mut |value| retain_string(value, name_formation.as_ref(), cancelled),
            )?;
            let position = self.attribute_position(
                element.attributes(),
                Some(name.namespace()),
                name.local_name(),
                cancelled,
            )?;
            (position, name)
        } else {
            if !valid_attribute_local_name(name, &mut || self.name_step(cancelled))? {
                return Err(DomError::InvalidCharacter);
            }
            let name = if self.html_element_in_html_document(node)? {
                crate::names::ascii_lowercase(
                    name,
                    &mut || self.name_step(cancelled),
                    &mut |input, lower| {
                        normalized.units(input, lower, name_formation.as_ref(), cancelled)
                    },
                    &mut |value| retain_string(value, name_formation.as_ref(), cancelled),
                )?
            } else {
                retain_string(name, name_formation.as_ref(), cancelled)?
            };
            let position = self.attribute_position(element.attributes(), None, &name, cancelled)?;
            (
                position,
                surgeist_html::values::QualifiedName::new(Namespace::none(), None, name),
            )
        };
        // Existing set-value preserves its exact canonical prefix and order.
        // Discard lookup spelling before admitting the selected occurrence.
        let name = if let Some(position) = position {
            drop(raw_name);
            drop(name_formation);
            name_formation = source::formation::Formation::new(self, cancelled)?;
            self.bind_canonical_name(
                element
                    .attributes()
                    .get(position)
                    .ok_or(DomError::StaleNode)?
                    .name(),
                name_formation.as_ref(),
                cancelled,
            )?
        } else {
            let canonical = source::formation::Formation::new(self, cancelled)?;
            let name = self.bind_name(&raw_name, canonical.as_ref(), cancelled)?;
            drop(raw_name);
            drop(name_formation);
            name_formation = canonical;
            name
        };
        drop(normalized);
        self.write_attribute(
            AttributeWrite {
                node,
                position,
                name,
                value,
                cause: AttributeCause::Change,
                name_formation,
            },
            limits,
            cancelled,
        )
    }
    /// Append exact validated component records; no QName reconstruction.
    pub(super) fn append_attribute(
        &mut self,
        node: Key,
        attribute: &surgeist_html::values::Attribute,
        cause: AttributeCause,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.node(node)?.element().ok_or(DomError::WrongNodeKind)?;
        let name_formation = source::formation::Formation::new(self, cancelled)?;
        let name = self.bind_name(attribute.name(), name_formation.as_ref(), cancelled)?;
        self.write_attribute(
            AttributeWrite {
                node,
                position: None,
                name,
                value: attribute.value(),
                cause,
                name_formation,
            },
            limits,
            cancelled,
        )
    }
    pub(super) fn append_canonical_attribute(
        &mut self,
        node: Key,
        attribute: &Attribute,
        cause: AttributeCause,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.node(node)?.element().ok_or(DomError::WrongNodeKind)?;
        let name_formation = source::formation::Formation::new(self, cancelled)?;
        let name =
            self.bind_canonical_name(attribute.name(), name_formation.as_ref(), cancelled)?;
        self.write_attribute(
            AttributeWrite {
                node,
                position: None,
                name,
                value: attribute.value(),
                cause,
                name_formation,
            },
            limits,
            cancelled,
        )
    }
    fn write_attribute(
        &mut self,
        write: AttributeWrite<'_>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let AttributeWrite {
            node,
            position,
            name,
            value,
            cause,
            name_formation,
        } = write;
        check_cancel(cancel)?;
        #[cfg(test)]
        transaction::queues::admission_observation::before_payload(self);
        let read = self.element(node)?;
        let element = read.language();
        let previous = read.attributes();
        let old = position
            .map(|index| previous.get(index).ok_or(DomError::StaleNode))
            .transpose()?;
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let old_attr_units = old.map_or(Ok(0), attribute_units)?;
        for _ in 0..surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND {
            self.source_step(cancel)?;
        }
        use surgeist_html::security::nonce::HtmlNonceAttributeEffect;
        let nonce = surgeist_html::security::nonce::nonce_attribute_change(
            element.name(),
            &name,
            Some(value),
        );
        let old_nonce_units = element.nonce().map_or(0, DocumentString::len);
        let nonce_units = match &nonce {
            HtmlNonceAttributeEffect::Ignore => old_nonce_units,
            HtmlNonceAttributeEffect::ClearSlot => 0,
            HtmlNonceAttributeEffect::CopyValue(value) => value.len(),
        };
        let next_attr_units = name_units(&name)
            .and_then(|units| units.checked_add(value.len()))
            .ok_or(DomError::RepresentationExhausted)?;
        let family_units = element
            .family_units()
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .units
            .checked_sub(old_attr_units)
            .and_then(|units| units.checked_add(next_attr_units))
            .ok_or(DomError::RepresentationExhausted)?;
        let minimum = units
            .checked_sub(old_nonce_units)
            .and_then(|units| units.checked_sub(family_units))
            .and_then(|units| units.checked_add(nonce_units))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(minimum, limits.units, DomResource::StringCodeUnits)?;
        let hook = source::formation::Formation::new(self, cancel)?;
        let changed = retain_name(&name, hook.as_ref(), cancel)?;
        let new_value = retain_attribute_value(self, value, hook.as_ref(), cancel)?;
        let old_value = old
            .map(|attr| retain_attribute_value(self, attr.value(), hook.as_ref(), cancel))
            .transpose()?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let nonce = nonce_after_effect(self, nonce, formation.as_ref(), cancel)?;
        let replacement =
            Attribute::from_parts(name, retain_attribute_value(self, value, name_formation.as_ref(), cancel)?);
        let index = position.unwrap_or(previous.len());
        let next = previous.edited(index, position.is_some(), Some(&replacement))?;
        let formed = {
            let mut admit = |usage| self.admit_custody_usage(usage, cancel);
            let context = ElementFormation::new(
                self.custody_registry(),
                &self.blocks,
                formation.as_ref(),
                cancel,
                &mut admit,
            );
            element.with_attribute_effect(&previous, &next, &changed, nonce, context)?
        };
        let final_units = units
            .checked_sub(old_units)
            .and_then(|units| units.checked_add(formed.value.units()?))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(final_units, limits.units, DomResource::StringCodeUnits)?;
        // A shrinking native slot can make room for the Attr payload. Apply
        // that reduction first when both versions cannot fit in the draft.
        let deferred_element = if units > limits.units {
            self.publish_element(node, old_units, formed, formation, limits, cancel)?;
            None
        } else {
            Some((formed, formation))
        };
        if let Some(position) = position {
            let attr = self
                .node(node)?
                .attribute_ids()
                .get(position)
                .ok_or(DomError::StaleNode)?;
            if !self.attribute_string_equal(
                old_value.as_ref().ok_or(DomError::StaleNode)?.code_units(),
                value.code_units(),
                cancel,
            )? {
                self.replace_attr_value_formed(attr, replacement, name_formation, cancel)?;
                self.units = if deferred_element.is_some() {
                    units
                } else {
                    final_units
                };
                self.advance_attribute_revision(node, cancel)?;
            }
        } else {
            let document = self.node(node)?.document();
            let attr = self.allocate_formed(
                NewNode::Attr(replacement),
                document,
                limits,
                cancel,
                name_formation,
            )?;
            if self.is_private(node) {
                self.mark_source_private(attr, cancel)?;
            }
            self.set_attr_owner(attr, Some(node), cancel)?;
            self.edit_attribute_order(node, index, None, Some(attr), cancel)?;
        }
        if let Some((formed, formation)) = deferred_element {
            self.publish_element(node, old_units, formed, formation, limits, cancel)?;
        }
        self.attribute_hooks(
            AttributeSuffix {
                subject: node,
                changed,
                old_value,
                new_value: Some(new_value),
                cause,
            },
            hook.map(source::formation::Formation::into_charge),
            limits,
            cancel,
        )
    }

    fn edit_attribute_order(
        &mut self,
        element: Key,
        position: usize,
        removed: Option<Key>,
        added: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let ids = self.node(element)?.attribute_ids();
        let length = ids.len();
        if position > length || removed.is_some() && ids.get(position) != removed {
            return Err(DomError::StaleNode);
        }
        let next = length
            .checked_sub(usize::from(removed.is_some()))
            .and_then(|size| size.checked_add(usize::from(added.is_some())))
            .ok_or(DomError::RepresentationExhausted)?;
        let mut order = self.effect_vec();
        self.select_reserve(&mut order, next, cancel)?;
        for (index, id) in ids.iter().enumerate() {
            self.source_step(cancel)?;
            if index == position {
                order.extend(added);
                if removed.is_some() {
                    continue;
                }
            }
            order.push(id);
        }
        if position == length {
            order.extend(added);
        }
        self.replace_attribute_order(
            element,
            order.into_values(),
            removed.as_slice(),
            Some(added.as_slice()),
            cancel,
        )
    }

    fn attribute_hooks(
        &mut self,
        hook: AttributeSuffix,
        payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = hook.subject;
        let changed = &hook.changed;
        self.inline_attribute(node, changed, hook.new_value.as_ref(), cancelled)?;
        let old_value = hook.old_value.as_ref();
        let new_value = hook.new_value.as_ref();
        let change = if new_value.is_none() {
            AttributeChange::Removed
        } else if old_value.is_some() {
            AttributeChange::Set
        } else {
            AttributeChange::Added
        };
        self.input_attribute_change(node, changed, change, hook.cause, limits, cancelled)?;
        if self.script_attribute_change(
            node,
            changed,
            old_value.is_some(),
            new_value.is_none(),
            cancelled,
        )? {
            self.push_effect_retained(
                select::Effect::AfterScriptAttribute(hook),
                self.effect_scoped,
                payload,
                cancelled,
            )?;
            self.script_post_connection(node, cancelled)?;
            return self.run_effects(limits, cancelled);
        }
        self.attribute_after_script(hook, payload, limits, cancelled)
    }
    pub(super) fn attribute_after_script(
        &mut self,
        hook: AttributeSuffix,
        payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = hook.subject;
        let changed = &hook.changed;
        let old_value = hook.old_value.as_ref();
        let new_value = hook.new_value.as_ref();
        let change = if new_value.is_none() {
            AttributeChange::Removed
        } else if old_value.is_some() {
            AttributeChange::Set
        } else {
            AttributeChange::Added
        };
        self.reflected_attribute_change(node, changed, cancelled)?;
        self.slot_attribute(node, changed, old_value, limits, cancelled)?;
        self.control_attribute_change(node, changed, old_value, cancelled)?;
        self.select_attribute(node, changed, change, limits, cancelled)?;
        self.details_attribute_change(node, changed, old_value, new_value, limits, cancelled)?;
        if let Some(operation) = self.dialog_attribute_operation(
            node,
            changed,
            old_value.is_some(),
            new_value.is_some(),
            cancelled,
        )? {
            self.retain_dialog_attribute_tail(hook, operation, payload, cancelled)?;
            self.run_effects(limits, cancelled)
        } else {
            self.attribute_suffix(hook, payload, limits, cancelled)
        }
    }
    fn retain_dialog_attribute_tail(
        &mut self,
        hook: AttributeSuffix,
        operation: surgeist_html::elements::dialog::HtmlDialogOperation,
        payload: Option<source::checkpoint::Charge>,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = hook.subject;
        self.push_effect_retained(
            select::Effect::AttributeSuffix(hook),
            self.effect_scoped,
            payload,
            cancelled,
        )?;
        self.schedule_dialog_hook(node, operation, cancelled)
    }
    pub(super) fn attribute_suffix(
        &mut self,
        hook: AttributeSuffix,
        _payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = hook.subject;
        let changed = &hook.changed;
        let old_value = hook.old_value.as_ref();
        let new_value = hook.new_value.as_ref();
        self.base_attribute_change(node, changed, old_value, new_value, limits, cancelled)?;
        self.image_attribute_change(node, changed, old_value, new_value, cancelled)?;
        self.style_attribute_change(node, changed, limits, cancelled)?;
        self.meta_attribute_changed(node, changed, limits, cancelled)?;
        self.hyperlink_attribute_change(node, changed, old_value, new_value, limits, cancelled)
    }
    pub(super) fn remove_attribute(
        &mut self,
        node: Key,
        namespace: Option<&Namespace>,
        name: &DocumentString,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        #[cfg(test)]
        transaction::queues::admission_observation::before_payload(self);
        let read = self.element(node)?;
        let element = read.language();
        let previous = read.attributes();
        let name_formation = source::formation::Formation::new(self, cancelled)?;
        let mut normalized = name_registry::Normalized::new(&self.names);
        let name = if namespace.is_none() && self.html_element_in_html_document(node)? {
            crate::names::ascii_lowercase(
                name,
                &mut || self.name_step(cancelled),
                &mut |input, lower| {
                    normalized.units(input, lower, name_formation.as_ref(), cancelled)
                },
                &mut |value| retain_string(value, name_formation.as_ref(), cancelled),
            )?
        } else {
            retain_string(name, name_formation.as_ref(), cancelled)?
        };
        let position = self.attribute_position(previous, namespace, &name, cancelled)?;
        drop(name);
        drop(normalized);
        drop(name_formation);
        let Some(position) = position else {
            return Ok(());
        };
        let removed = previous.get(position).ok_or(DomError::StaleNode)?;
        let hook = source::formation::Formation::new(self, cancelled)?;
        let changed = retain_name(removed.name(), hook.as_ref(), cancelled)?;
        let old_value = retain_attribute_value(self, removed.value(), hook.as_ref(), cancelled)?;
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let candidate = source::formation::Formation::new(self, cancelled)?;
        for _ in 0..surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND {
            self.source_step(cancelled)?;
        }
        let nonce_value =
            surgeist_html::security::nonce::nonce_attribute_change(element.name(), &changed, None);
        let nonce = nonce_after_effect(self, nonce_value, candidate.as_ref(), cancelled)?;
        let next = previous.edited(position, true, None)?;
        let staged = {
            let mut admit = |usage| self.admit_custody_usage(usage, cancelled);
            let context = ElementFormation::new(
                self.custody_registry(),
                &self.blocks,
                candidate.as_ref(),
                cancelled,
                &mut admit,
            );
            element.with_attribute_effect(&previous, &next, &changed, nonce, context)?
        };
        let attr = self
            .node(node)?
            .attribute_ids()
            .get(position)
            .ok_or(DomError::StaleNode)?;
        self.set_attr_owner(attr, None, cancelled)?;
        self.edit_attribute_order(node, position, Some(attr), None, cancelled)?;
        self.publish_element(node, old_units, staged, candidate, limits, cancelled)?;
        self.attribute_hooks(
            AttributeSuffix {
                subject: node,
                changed,
                old_value: Some(old_value),
                new_value: None,
                cause: AttributeCause::Change,
            },
            hook.map(source::formation::Formation::into_charge),
            limits,
            cancelled,
        )
    }
    fn publish_element(
        &mut self,
        node: Key,
        old_units: usize,
        staged: FormedElement,
        formation: Option<source::formation::Formation>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        struct Candidate {
            value: FormedElement,
            formation: Option<source::formation::Formation>,
        }
        let candidate = Candidate {
            value: staged,
            formation,
        };
        let units = self
            .units
            .checked_sub(old_units)
            .and_then(|units| units.checked_add(candidate.value.value.units()?))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let Candidate { value, formation } = candidate;
        self.replace_element_formed(node, value, formation, cancelled)?;
        self.units = units;
        Ok(())
    }
}
fn nonce_after_effect(
    draft: &Draft,
    effect: surgeist_html::security::nonce::HtmlNonceAttributeEffect<'_>,
    budget: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<NonceUpdate, DomError> {
    use surgeist_html::security::nonce::HtmlNonceAttributeEffect;
    match effect {
        HtmlNonceAttributeEffect::Ignore => Ok(NonceUpdate::Unchanged),
        HtmlNonceAttributeEffect::ClearSlot => {
            source::formation::units(&[], false, budget, cancel).map(NonceUpdate::Set)
        }
        HtmlNonceAttributeEffect::CopyValue(value) => {
            retain_attribute_value(draft, value, budget, cancel).map(NonceUpdate::Set)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_preserves_prefix_order_nonce_and_historical_utf16() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                64,
                0,
                0,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        let associated = setup.create_document();
        let document = setup.create_html_document(&associated, None);
        let element = setup.create_element_ns(&document, Namespace::html(), "div");
        let namespace = Namespace::from_uri("urn:attributes");
        setup.set_attribute_ns(&element, namespace.clone(), "first:item", "old");
        setup.set_attribute(&element, "TaIl", "last");
        setup.set_nonce(&element, "private");
        let commit = dom.apply(setup, &cancel).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        let old = dom.snapshot();
        let value = DocumentString::from_utf16(vec![0xd800, 0x41, 0xdc00]);
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(element, namespace.clone(), "second:item", value.clone());
        batch.set_attribute(element, "NoNcE", "new nonce");
        let limits =
            crate::DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let mut lease = dom.begin_source_commands(batch, limits, &cancel).unwrap();
        assert!(matches!(
            lease.advance(&cancel),
            Ok(crate::DomSourceProgress::Complete(_))
        ));
        drop(lease);
        let current = dom.snapshot();
        let changed = current.node(element).unwrap();
        assert_eq!(
            changed
                .attributes()
                .unwrap()
                .item(0)
                .unwrap()
                .name()
                .prefix(),
            Some(&"first".into())
        );
        assert_eq!(
            changed.attributes().unwrap().item(0).unwrap().value(),
            &value
        );
        assert_eq!(
            changed
                .attributes()
                .unwrap()
                .item(1)
                .unwrap()
                .name()
                .local_name(),
            &"tail".into()
        );
        assert_eq!(
            changed.as_element().unwrap().nonce(),
            Some(&"new nonce".into())
        );
        let historical = old.node(element).unwrap();
        assert_eq!(
            historical.attributes().unwrap().item(0).unwrap().value(),
            &"old".into()
        );
        assert_eq!(
            historical.as_element().unwrap().nonce(),
            Some(&"private".into())
        );
        let mut remove = DomBatch::new();
        remove.remove_attribute_ns(element, namespace, "item");
        remove.remove_attribute(element, "nonce");
        dom.apply(remove, &cancel).unwrap();
        let removed = dom.snapshot();
        let removed = removed.node(element).unwrap();
        assert_eq!(removed.attributes().unwrap().len(), 1);
        assert_eq!(
            removed
                .attributes()
                .unwrap()
                .item(0)
                .unwrap()
                .name()
                .local_name(),
            &"tail".into()
        );
        assert_eq!(removed.as_element().unwrap().nonce(), Some(&"".into()));
    }
    #[test]
    fn name_errors_absent_removal_and_late_failure_preserve_atomic_state() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                64,
                0,
                0,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let element = setup.create_element_ns(&document, Namespace::html(), "div");
        setup.set_attribute(&element, "kept", "old");
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        let old = dom.snapshot();
        for (name, expected) in [
            ("p:bad=name", DomError::InvalidCharacter),
            ("p:local", DomError::Namespace),
        ] {
            let mut batch = DomBatch::new();
            batch.set_attribute_ns(element, Namespace::none(), name, "ignored");
            assert_eq!(dom.apply(batch, &cancel).unwrap_err(), expected);
            assert_eq!(dom.snapshot().stamp(), old.stamp());
        }
        let mut absent = DomBatch::new();
        absent.remove_attribute(element, "bad=name");
        dom.apply(absent, &cancel).unwrap();
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        let mut late = DomBatch::new();
        late.set_attribute(element, "kept", "new");
        late.set_attribute(document, "wrong-kind", "x");
        let limits =
            crate::DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let mut lease = dom.begin_source_commands(late, limits, &cancel).unwrap();
        assert_eq!(lease.advance(&cancel).unwrap_err(), DomError::WrongNodeKind);
        assert_eq!(lease.snapshot().stamp(), old.stamp());
        drop(lease);
        let mut cancelled = DomBatch::new();
        cancelled.set_attribute(element, "kept", "cancelled");
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            dom.apply(cancelled, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
    }
}

#[cfg(test)]
mod dialog_tail_tests {
    use super::*;

    #[test]
    fn actual_dialog_attribute_tail_preserves_owned_payload_through_checkpoint_and_cancel() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                16384,
                64,
                0,
                0,
                16,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
        batch.set_attribute(&dialog, "open", "kept");
        let commit = dom.apply(batch, &cancel).unwrap();
        let dialog = commit.created(&dialog).unwrap().unwrap();
        let stamp = dom.snapshot().stamp();
        let batch = DomBatch::new();
        let limits =
            crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
        let (mut draft, _) =
            Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel).unwrap();
        let key = draft.resolve(dialog.into()).unwrap();
        let element = draft.element(key).unwrap();
        let attribute = element.attributes().get(0).unwrap();
        let formation = source::formation::Formation::new(&draft, &cancel).unwrap();
        let changed = retain_name(attribute.name(), formation.as_ref(), &cancel).unwrap();
        let new_value = retain_string(attribute.value(), formation.as_ref(), &cancel).unwrap();
        // Real immutable operand occurrences, independently priced from their
        // actual UTF16 banks; inline frame fields belong to the effect bank.
        let bytes = 2 * size_of::<Vec<u16>>()
            + (changed.local_name().retained_units() + new_value.retained_units())
                * size_of::<u16>();
        let units = changed.local_name().len() + new_value.len();
        let operation = draft
            .dialog_attribute_operation(key, &changed, false, true, &cancel)
            .unwrap()
            .unwrap();
        draft
            .retain_dialog_attribute_tail(
                AttributeSuffix {
                    subject: key,
                    changed,
                    old_value: None,
                    new_value: Some(new_value),
                    cause: AttributeCause::Change,
                },
                operation,
                formation.map(source::formation::Formation::into_charge),
                &cancel,
            )
            .unwrap();
        assert_eq!(draft.effects.len(), 2);
        assert!(matches!(
            draft.effects[0].effect,
            select::Effect::AttributeSuffix(_)
        ));
        assert!(matches!(
            draft.effects[1].effect,
            select::Effect::DialogHook(_)
        ));
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), bytes);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), units);
        let checkpoint = draft
            .source_checkpoint(draft.source_other_bytes, 0, &cancel)
            .unwrap();
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert!(draft.effects.is_empty());
        checkpoint.restore(&mut draft);
        assert_eq!(draft.effects.len(), 2);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), bytes);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), units);
        assert_eq!(
            draft.run_effects(dom.limits, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        );
        assert_eq!(draft.effects.len(), 2);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), bytes);
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert!(draft.effects.is_empty());
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        assert_eq!(
            draft
                .element(key)
                .unwrap()
                .attributes()
                .iter()
                .find(|attr| attr.name().local_name() == &DocumentString::from("open"))
                .map(crate::DomAttribute::value),
            Some(&"kept".into())
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
        // This native inactive executor test supplies no watcher completion or
        // hosted activity. The public Source test owns that actual wait proof.
    }
}
