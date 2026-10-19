use super::*;
use crate::element::{ElementFormation, FormedElement, NonceUpdate};
use crate::names::{attribute_name, name_units, valid_attribute_local_name};
use crate::{DomAttribute as Attribute, DomQualifiedName as QualifiedName, Namespace};

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
            payload.string(value)?;
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
        attributes: &[Attribute],
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
        let value = retain_string(value, candidate.as_ref(), cancelled)?;
        let mut original = source::storage::Payload::default();
        if candidate.is_some() {
            original.string(&value)?;
        }
        let (value, controls) = {
            let mut admit = |next| self.admit_custody_usage(next, cancelled);
            let mut context = ElementFormation::new(
                self.custody_registry(),
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
        let element = self
            .node(node)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .language();
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
                crate::QualifiedName::new(Namespace::none(), None, name),
            )
        };
        // Existing set-value preserves its exact canonical prefix and order.
        // Discard lookup spelling before admitting the selected occurrence.
        let name = if let Some(position) = position {
            drop(raw_name);
            drop(name_formation);
            name_formation = source::formation::Formation::new(self, cancelled)?;
            self.bind_canonical_name(
                element.attributes()[position].name(),
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
        attribute: &crate::Attribute,
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
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = write.node;
        let position = write.position;
        let value = write.value;
        let cause = write.cause;
        check_cancel(cancelled)?;
        #[cfg(test)]
        transaction::queues::admission_observation::before_payload(self);
        let element = self
            .node(node)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .language();
        #[cfg(test)]
        let copy_boundary = attribute_copy_observation::before(
            self,
            &write.name,
            write.name_formation.is_some(),
            cancelled,
        )?;
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let old_attribute =
            position.map_or(Ok(0), |index| attribute_units(&element.attributes()[index]))?;
        let base = self.units
            - old_attribute
            - element.nonce().map_or(0, DocumentString::len)
            - element
                .family_units()
                .ok_or(DomError::RepresentationExhausted)?;
        for _ in 0..surgeist_html::bounded::HTML_NONCE_INSPECTION_BOUND {
            self.source_step(cancelled)?;
        }
        let nonce_value =
            surgeist_html::nonce_attribute_change(element.name(), &write.name, Some(value));
        let nonce_units = match nonce_value {
            surgeist_html::HtmlNonceAttributeEffect::Ignore => {
                element.nonce().map_or(0, DocumentString::len)
            }
            surgeist_html::HtmlNonceAttributeEffect::ClearSlot => 0,
            surgeist_html::HtmlNonceAttributeEffect::CopyValue(value) => value.len(),
        };
        let minimum = base
            .checked_add(name_units(&write.name).ok_or(DomError::RepresentationExhausted)?)
            .and_then(|n| n.checked_add(value.len()))
            .and_then(|n| n.checked_add(nonce_units))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(minimum, limits.units, DomResource::StringCodeUnits)?;
        let hook = source::formation::Formation::new(self, cancelled)?;
        let changed = retain_name(&write.name, hook.as_ref(), cancelled)?;
        let details_new_value = retain_string(value, hook.as_ref(), cancelled)?;
        let old_value = position
            .map(|index| {
                retain_string(
                    element.attributes()[index].value(),
                    hook.as_ref(),
                    cancelled,
                )
            })
            .transpose()?;
        let candidate = source::formation::Formation::new(self, cancelled)?;
        let replacement = Attribute::from_parts(
            write.name,
            retain_string(value, candidate.as_ref(), cancelled)?,
        );
        let nonce = nonce_after_effect(nonce_value, candidate.as_ref(), cancelled)?;
        #[cfg(test)]
        let mut attribute_writes = crate::store::CopyObservation::default();
        let capacity = element
            .attributes()
            .len()
            .checked_add(usize::from(position.is_none()))
            .ok_or(DomError::RepresentationExhausted)?;
        let mut attributes =
            self.attribute_input(capacity, candidate.as_ref(), cancelled, |values| {
                source::formation::reserve(values, capacity, candidate.as_ref(), cancelled)
            })?;
        for (index, attribute) in element.attributes().iter().enumerate() {
            self.source_step(cancelled)?;
            if Some(index) == position {
                continue;
            }
            let copied = Attribute::from_parts(
                retain_name(attribute.name(), candidate.as_ref(), cancelled)?,
                retain_string(attribute.value(), candidate.as_ref(), cancelled)?,
            );
            attributes.entries.push(copied);
            #[cfg(test)]
            attribute_writes.pushed::<Attribute>();
        }
        if let Some(position) = position {
            attributes.entries.insert(position, replacement);
            #[cfg(test)]
            attribute_writes.extended::<Attribute>(attributes.entries.len() - position);
        } else {
            attributes.entries.push(replacement);
            #[cfg(test)]
            attribute_writes.pushed::<Attribute>();
        }
        #[cfg(test)]
        attribute_copy_observation::copied(copy_boundary, &attributes.entries, self, cancelled)?;
        if let Some(name_formation) = write.name_formation {
            candidate
                .as_ref()
                .expect("same Source payload producer")
                .absorb(name_formation);
        }
        let attributes = self.form_attributes(attributes, candidate.as_ref(), cancelled)?;
        let attributes = self.form_attribute_binding(attributes, candidate.as_ref(), cancelled)?;
        let (binding, binding_custody) = match attributes {
            Some(formed) => (Some(formed.value), Some(formed.custody)),
            None => (None, None),
        };
        let mut staged = {
            let mut admit = |next| self.admit_custody_usage(next, cancelled);
            let context = ElementFormation::new(
                self.custody_registry(),
                candidate.as_ref(),
                cancelled,
                &mut admit,
            );
            element.with_attributes_formed(binding, &changed, nonce, context)?
        };
        staged.children.attributes = binding_custody;
        self.publish_element(node, old_units, staged, candidate, limits, cancelled)?;
        #[cfg(test)]
        attribute_copy_observation::hooks(
            &changed,
            old_value.as_ref(),
            Some(&details_new_value),
            self,
        );
        let result = self.attribute_hooks(
            AttributeSuffix {
                subject: node,
                changed,
                old_value,
                new_value: Some(details_new_value),
                cause,
            },
            hook.map(source::formation::Formation::into_charge),
            limits,
            cancelled,
        );
        #[cfg(test)]
        attribute_copy_observation::released(self);
        result
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
        self.refuse_inline_attribute(node, changed, cancelled)?;
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
        self.script_attribute_change(
            node,
            changed,
            old_value.is_some(),
            new_value.is_none(),
            cancelled,
        )?;
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
        operation: surgeist_html::HtmlDialogOperation,
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
        let element = self
            .node(node)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .language();
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
        let position =
            self.attribute_position(element.attributes(), namespace, &name, cancelled)?;
        drop(name);
        drop(normalized);
        drop(name_formation);
        let Some(position) = position else {
            return Ok(());
        };
        let removed = &element.attributes()[position];
        #[cfg(test)]
        let copy_boundary =
            attribute_copy_observation::before(self, removed.name(), false, cancelled)?;
        let hook = source::formation::Formation::new(self, cancelled)?;
        let changed = retain_name(removed.name(), hook.as_ref(), cancelled)?;
        let old_value = retain_string(removed.value(), hook.as_ref(), cancelled)?;
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let candidate = source::formation::Formation::new(self, cancelled)?;
        for _ in 0..surgeist_html::bounded::HTML_NONCE_INSPECTION_BOUND {
            self.source_step(cancelled)?;
        }
        let nonce_value = surgeist_html::nonce_attribute_change(element.name(), &changed, None);
        let nonce = nonce_after_effect(nonce_value, candidate.as_ref(), cancelled)?;
        #[cfg(test)]
        let mut attribute_writes = crate::store::CopyObservation::default();
        let capacity = element.attributes().len();
        let mut attributes =
            self.attribute_input(capacity, candidate.as_ref(), cancelled, |values| {
                source::formation::reserve(values, capacity, candidate.as_ref(), cancelled)
            })?;
        for (index, attribute) in element.attributes().iter().enumerate() {
            self.source_step(cancelled)?;
            if index == position {
                continue;
            }
            attributes.entries.push(Attribute::from_parts(
                retain_name(attribute.name(), candidate.as_ref(), cancelled)?,
                retain_string(attribute.value(), candidate.as_ref(), cancelled)?,
            ));
            #[cfg(test)]
            attribute_writes.pushed::<Attribute>();
        }
        #[cfg(test)]
        attribute_copy_observation::copied(copy_boundary, &attributes.entries, self, cancelled)?;
        let attributes = self.form_attributes(attributes, candidate.as_ref(), cancelled)?;
        let attributes = self.form_attribute_binding(attributes, candidate.as_ref(), cancelled)?;
        let (binding, binding_custody) = match attributes {
            Some(formed) => (Some(formed.value), Some(formed.custody)),
            None => (None, None),
        };
        let mut staged = {
            let mut admit = |next| self.admit_custody_usage(next, cancelled);
            let context = ElementFormation::new(
                self.custody_registry(),
                candidate.as_ref(),
                cancelled,
                &mut admit,
            );
            element.with_attributes_formed(binding, &changed, nonce, context)?
        };
        staged.children.attributes = binding_custody;
        self.publish_element(node, old_units, staged, candidate, limits, cancelled)?;
        #[cfg(test)]
        attribute_copy_observation::hooks(&changed, Some(&old_value), None, self);
        let result = self.attribute_hooks(
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
        );
        #[cfg(test)]
        attribute_copy_observation::released(self);
        result
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
        let units = (self.units - old_units)
            .checked_add(
                candidate
                    .value
                    .value
                    .units()
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let Candidate { value, formation } = candidate;
        self.replace_element_formed(node, value, formation, cancelled)?;
        self.units = units;
        Ok(())
    }
}
fn nonce_after_effect(
    effect: surgeist_html::HtmlNonceAttributeEffect<'_>,
    budget: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<NonceUpdate, DomError> {
    use surgeist_html::HtmlNonceAttributeEffect;
    match effect {
        HtmlNonceAttributeEffect::Ignore => Ok(NonceUpdate::Unchanged),
        HtmlNonceAttributeEffect::ClearSlot => {
            source::formation::units(&[], false, budget, cancel).map(NonceUpdate::Set)
        }
        HtmlNonceAttributeEffect::CopyValue(value) => {
            retain_string(value, budget, cancel).map(NonceUpdate::Set)
        }
    }
}

#[cfg(test)]
mod attribute_copy_observation {
    use super::*;
    use std::cell::Cell;
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Boundary {
        pub required: usize,
        pub limit: usize,
        pub admitted: usize,
        pub admitted_units: usize,
        pub base_live: usize,
        pub hook_expected: usize,
        pub hook_live: usize,
        pub released_live: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Boundary>> = const { Cell::new(None) };
        static CANCEL_AFTER_COPY: Cell<bool> = const { Cell::new(false) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
        CANCEL_AFTER_COPY.set(false);
    }
    pub(super) fn stop_after_copy() {
        CANCEL_AFTER_COPY.set(true);
    }
    pub(super) fn finish() -> Option<Boundary> {
        ACTIVE.set(false);
        CANCEL_AFTER_COPY.set(false);
        LAST.take()
    }
    pub(super) fn before(
        draft: &Draft,
        name: &QualifiedName,
        registered: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Boundary>, DomError> {
        if !ACTIVE.get() {
            return Ok(None);
        }
        let Some(limits) = draft.source_limits else {
            return Ok(None);
        };
        // The replacement QName is already retained by its exact Formation
        // after the correction. It is part of the observed new attribute graph,
        // so remove that registered contribution before adding the whole graph.
        let mut registered_bytes = 0;
        if registered {
            for string in [name.namespace_uri(), name.prefix(), Some(name.local_name())]
                .into_iter()
                .flatten()
            {
                registered_bytes += size_of::<Vec<u16>>()
                    + string.shared_code_units().capacity() * size_of::<u16>();
            }
        }
        Ok(Some(Boundary {
            required: draft.source_storage_bytes(cancel)? + draft.source_other_bytes
                - registered_bytes,
            limit: limits.retained_bytes,
            admitted: 0,
            admitted_units: 0,
            base_live: draft.source_formation_bytes.load(Ordering::Relaxed) - registered_bytes,
            hook_expected: 0,
            hook_live: 0,
            released_live: usize::MAX,
        }))
    }
    pub(super) fn copied(
        before: Option<Boundary>,
        attributes: &Vec<Attribute>,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(mut boundary) = before {
            // The actual completed input buffer and retained occurrences, plus
            // the target body already reserved before that buffer's allocation.
            // Sealing the target/child custody follows this raw-copy window.
            boundary.required += size_of::<payload::Payload<crate::name::AttributeValues>>()
                + attributes.capacity() * size_of::<Attribute>();
            for attr in attributes.iter() {
                for string in [
                    attr.name().namespace_uri(),
                    attr.name().prefix(),
                    Some(attr.name().local_name()),
                    Some(attr.value()),
                ]
                .into_iter()
                .flatten()
                {
                    boundary.required += size_of::<Vec<u16>>()
                        + string.shared_code_units().capacity() * size_of::<u16>();
                }
            }
            boundary.admitted = draft.source_storage_bytes(cancel)? + draft.source_other_bytes;
            boundary.admitted_units =
                draft.source_storage_units(cancel)? + draft.source_other_units;
            LAST.set(Some(boundary));
            if CANCEL_AFTER_COPY.get() {
                // Stop at the actual completed copy, before its Formation is
                // reused by the separate element/family publication tail.
                cancel.store(true, Ordering::Release);
                check_cancel(cancel)?;
            }
        }
        Ok(())
    }
    pub(super) fn hooks(
        name: &QualifiedName,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
        draft: &Draft,
    ) {
        if let Some(mut boundary) = LAST.get() {
            let mut expected = boundary.base_live;
            for string in [
                name.namespace_uri(),
                name.prefix(),
                Some(name.local_name()),
                old,
                new,
            ]
            .into_iter()
            .flatten()
            {
                expected += size_of::<Vec<u16>>()
                    + string.shared_code_units().capacity() * size_of::<u16>();
            }
            boundary.hook_expected = expected;
            boundary.hook_live = draft.source_formation_bytes.load(Ordering::Relaxed);
            LAST.set(Some(boundary));
        }
    }
    pub(super) fn released(draft: &Draft) {
        if let Some(mut boundary) = LAST.get() {
            boundary.released_live = draft.source_formation_bytes.load(Ordering::Relaxed);
            LAST.set(Some(boundary));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn copied_additional(
        element: &DomElement,
        remove: bool,
        new: &DocumentString,
    ) -> custody::Usage {
        fn string(value: &DocumentString, bytes: &mut usize, units: &mut usize) {
            let capacity = value.shared_code_units().capacity();
            *bytes += size_of::<Vec<u16>>() + capacity * size_of::<u16>();
            *units += capacity;
        }
        fn name(value: &QualifiedName, bytes: &mut usize, units: &mut usize) {
            for value in [
                value.namespace_uri(),
                value.prefix(),
                Some(value.local_name()),
            ]
            .into_iter()
            .flatten()
            {
                string(value, bytes, units);
            }
        }
        let attributes = element.attributes();
        let last = attributes.last().unwrap();
        // The replacement requests the existing length; old spare capacity
        // belongs to the already-live bank and is not copied into the new one.
        let mut bytes = size_of::<payload::Payload<crate::name::AttributeValues>>()
            + std::mem::size_of_val(attributes);
        let mut units = 0;
        // This unrelated data-* edit aliases the original nonce group.
        // Ignore forms no new nonce occurrence at this raw-copy boundary.
        for attribute in &attributes[..attributes.len() - 1] {
            name(attribute.name(), &mut bytes, &mut units);
            string(attribute.value(), &mut bytes, &mut units);
        }
        // Hook QName/old value and candidate strings are separate occurrences.
        // A replacement QName is already live at write_attribute entry.
        name(last.name(), &mut bytes, &mut units);
        string(last.value(), &mut bytes, &mut units);
        if !remove {
            string(new, &mut bytes, &mut units);
            string(new, &mut bytes, &mut units);
        }
        custody::Usage::new(bytes, units)
    }
    fn actual_attribute_copy_with(
        bytes: usize,
        units: usize,
        append: bool,
        remove: bool,
    ) -> Option<attribute_copy_observation::Boundary> {
        actual_attribute_copy_selected_with(bytes, units, append, remove, None)
    }
    fn actual_attribute_copy_selected_with(
        bytes: usize,
        units: usize,
        append: bool,
        remove: bool,
        selected: Option<(DomResource, bool)>,
    ) -> Option<attribute_copy_observation::Boundary> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                2_000_000,
                128,
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
        for index in 0..32 {
            let mut units = Vec::with_capacity(16_384);
            units.push(0xd800);
            setup.set_attribute(
                &element,
                format!("data-{index}"),
                DocumentString::from_utf16(units),
            );
        }
        let commit = dom.apply(setup, &cancel).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        let before = dom.snapshot();
        assert_eq!(
            before.node(element).unwrap().as_element().unwrap().nonce(),
            Some(&DocumentString::default())
        );
        let new_value = DocumentString::from("new");
        let additional = copied_additional(
            before.node(element).unwrap().as_element().unwrap(),
            remove,
            &new_value,
        );
        let mut batch = DomBatch::new();
        if remove {
            batch.remove_attribute(element, "data-31");
        } else {
            batch.set_attribute(
                element,
                if append { "data-new" } else { "data-31" },
                new_value,
            );
        }
        let limits = crate::DomSourceLimits::new(bytes, units, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limits, &cancel) else {
            return None;
        };
        let boundary = selected.map(|(resource, short)| {
            transaction::queues::admission_observation::Binding::payload(
                additional, resource, short,
            )
        });
        attribute_copy_observation::start();
        if selected.is_some() {
            attribute_copy_observation::stop_after_copy();
        }
        let result = lease.advance(&cancel);
        let observed = attribute_copy_observation::finish();
        if let Some(boundary) = boundary {
            let limit = boundary.limit();
            let (resource, short) = selected.unwrap();
            if short {
                assert!(
                    matches!(&result, Err(DomError::ResourceLimit { resource: actual, limit: actual_limit }) if *actual == resource && *actual_limit == limit),
                    "selected {resource:?} limit {limit}, actual {:?}",
                    result.as_ref().err()
                );
                assert!(
                    observed.is_none(),
                    "one short stops before the completed copy"
                );
            } else {
                assert!(
                    matches!(result, Err(DomError::Cancelled)),
                    "exact selected copy reaches the deliberate post-copy cancellation: {:?}",
                    result.as_ref().err()
                );
                assert!(
                    observed.is_some(),
                    "the real raw copy completed before cancellation"
                );
            }
            drop(boundary);
        }
        if result.is_err() {
            let restored = lease.snapshot();
            assert_eq!(restored.stamp(), before.stamp());
            assert_eq!(*restored.state, *before.state);
            let node = restored.node(element).unwrap();
            let attributes = node.as_element().unwrap().attributes();
            assert_eq!(attributes.len(), 32);
            assert_eq!(attributes[31].value().code_units(), &[0xd800]);
            assert_eq!(
                node.as_element().unwrap().nonce(),
                Some(&DocumentString::default())
            );
        }
        observed
    }
    fn actual_attribute_copy(bytes: usize) -> Option<attribute_copy_observation::Boundary> {
        actual_attribute_copy_with(bytes, 10_000_000, true, false)
    }
    #[test]
    fn source_attribute_copy_admits_actual_bank_and_backings_before_copy() {
        let high = actual_attribute_copy(64_000_000).expect("actual attribute bank copy");
        if let Some(actual) = actual_attribute_copy(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "attribute bank retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    fn assert_copy_envelope(remove: bool) {
        for resource in [
            DomResource::SourceRetainedBytes,
            DomResource::SourceRetainedUnits,
        ] {
            let exact = actual_attribute_copy_selected_with(
                64_000_000,
                10_000_000,
                false,
                remove,
                Some((resource, false)),
            )
            .expect("exact selected copy is reachable");
            assert!(exact.required <= exact.limit);
            assert!(
                actual_attribute_copy_selected_with(
                    64_000_000,
                    10_000_000,
                    false,
                    remove,
                    Some((resource, true))
                )
                .is_none()
            );
        }
    }
    fn assert_hook_lifetime(remove: bool) {
        let actual = actual_attribute_copy_with(64_000_000, 10_000_000, false, remove)
            .expect("actual mutation");
        assert_eq!(
            actual.hook_live, actual.hook_expected,
            "candidate/name/old family occurrences must end at canonical transfer"
        );
        assert_eq!(
            actual.released_live, actual.base_live,
            "hook copies must release after actual callbacks"
        );
    }
    #[test]
    fn source_replacement_raw_copy_has_exact_byte_and_utf16_boundaries() {
        assert_copy_envelope(false);
    }
    #[test]
    fn source_hooks_keep_only_their_actual_copies_then_release_them() {
        assert_hook_lifetime(false);
    }
    #[test]
    fn source_removal_admits_exact_copy_and_releases_hook_custody() {
        assert_copy_envelope(true);
        assert_hook_lifetime(true);
    }
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
        let changed = current.node(element).unwrap().as_element().unwrap();
        assert_eq!(
            changed.attributes()[0].name().prefix(),
            Some(&"first".into())
        );
        assert_eq!(changed.attributes()[0].value(), &value);
        assert_eq!(changed.attributes()[1].name().local_name(), &"tail".into());
        assert_eq!(changed.nonce(), Some(&"new nonce".into()));
        let historical = old.node(element).unwrap().as_element().unwrap();
        assert_eq!(historical.attributes()[0].value(), &"old".into());
        assert_eq!(historical.nonce(), Some(&"private".into()));
        let mut remove = DomBatch::new();
        remove.remove_attribute_ns(element, namespace, "item");
        remove.remove_attribute(element, "nonce");
        dom.apply(remove, &cancel).unwrap();
        let removed = dom.snapshot();
        let removed = removed.node(element).unwrap().as_element().unwrap();
        assert_eq!(removed.attributes().len(), 1);
        assert_eq!(removed.attributes()[0].name().local_name(), &"tail".into());
        assert_eq!(removed.nonce(), Some(&"".into()));
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
        let attribute = &element.attributes()[0];
        let formation = source::formation::Formation::new(&draft, &cancel).unwrap();
        let changed = retain_name(attribute.name(), formation.as_ref(), &cancel).unwrap();
        let new_value = retain_string(attribute.value(), formation.as_ref(), &cancel).unwrap();
        // Real immutable operand occurrences, independently priced from their
        // actual UTF16 banks; inline frame fields belong to the effect bank.
        let bytes = 2 * size_of::<Vec<u16>>()
            + (changed.local_name().shared_code_units().capacity()
                + new_value.shared_code_units().capacity())
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
            draft.element(key).unwrap().attribute(&"open".into()),
            Some(&"kept".into())
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
        // This native inactive executor test supplies no watcher completion or
        // hosted activity. The public Source test owns that actual wait proof.
    }
}
