use super::*;
use crate::names::{attribute_name, name_units, valid_attribute_local_name};
use crate::{Attribute, Namespace, OrderedAttributes, QualifiedName};
use surgeist_html::bounded::{HtmlAttributeStorage, HtmlAttributeValidation};

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
    name_formation: Option<style::formation::Formation>,
}
struct HookChange<'a> {
    changed: &'a QualifiedName,
    old_value: Option<&'a DocumentString>,
    new_value: Option<&'a DocumentString>,
    cause: AttributeCause,
}

fn attribute_units(attribute: &Attribute) -> Result<usize, DomError> {
    name_units(attribute.name())
        .and_then(|units| units.checked_add(attribute.value().len()))
        .ok_or(DomError::RepresentationExhausted)
}

fn retain_string(
    value: &DocumentString,
    budget: Option<&style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    if let Some(budget) = budget {
        budget.step(cancel)?;
        let mut payload = style::storage::Payload::default();
        payload.string(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}
fn retain_name(
    value: &QualifiedName,
    budget: Option<&style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<QualifiedName, DomError> {
    if let Some(budget) = budget {
        budget.step(cancel)?;
        let mut payload = style::storage::Payload::default();
        payload.name(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}
fn publish_attributes(
    attributes: Vec<Attribute>,
    budget: Option<&style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<OrderedAttributes, DomError> {
    if let Some(budget) = budget {
        budget.retain_bytes(size_of::<Vec<Attribute>>(), cancel)?;
    }
    let mut validation =
        HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes)).validate();
    loop {
        style::formation::step(budget, cancel)?;
        let Some(action) = validation.next_action().expect("unique expanded names") else {
            break;
        };
        let layout = action.layout();
        let work = layout
            .record_visits()
            .checked_add(layout.inspected_bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        for _ in 0..work {
            style::formation::step(budget, cancel)?;
        }
        action
            .advance()
            .expect("source attribute identity preserved");
    }
    let validated = HtmlAttributeValidation::try_finish(&mut Some(validation))
        .expect("completed unique attributes");
    Ok(validated.into_attributes())
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
            self.source_step(cancel)?;
            if left != right {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn attribute_namespace_equal(
        &self,
        left: &Namespace,
        right: &Namespace,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        match (left.uri(), right.uri()) {
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
            self.source_step(cancel)?;
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
        attributes: &OrderedAttributes,
        namespace: Option<&Namespace>,
        name: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<Option<usize>, DomError> {
        for (position, attribute) in attributes.iter().enumerate() {
            self.source_step(cancel)?;
            let matched = if let Some(namespace) = namespace {
                self.attribute_namespace_equal(attribute.name().namespace(), namespace, cancel)?
                    && self.attribute_string_equal(
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
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let old = element.nonce().ok_or(DomError::NonceInapplicable)?.len();
        let units = (self.units - old)
            .checked_add(value.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let candidate = style::formation::Formation::new(self, cancelled)?;
        let value = retain_string(value, candidate.as_ref(), cancelled)?;
        let DomNodeKind::Element(element) = &mut self.node_mut(node).kind else {
            unreachable!("checked Element");
        };
        element.set_nonce(value);
        self.units = units;
        drop(candidate);
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
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let mut name_formation = style::formation::Formation::new(self, cancelled)?;
        let (position, mut name) = if let Some(namespace) = namespace {
            let name = attribute_name(
                namespace,
                name,
                &mut || self.source_step(cancelled),
                &mut |input, lower| {
                    style::formation::units(input, lower, name_formation.as_ref(), cancelled)
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
            if !valid_attribute_local_name(name, &mut || self.source_step(cancelled))? {
                return Err(DomError::InvalidCharacter);
            }
            let name = if self.html_element_in_html_document(node)? {
                crate::names::ascii_lowercase(
                    name,
                    &mut || self.source_step(cancelled),
                    &mut |input, lower| {
                        style::formation::units(input, lower, name_formation.as_ref(), cancelled)
                    },
                    &mut |value| retain_string(value, name_formation.as_ref(), cancelled),
                )?
            } else {
                retain_string(name, name_formation.as_ref(), cancelled)?
            };
            let position = self.attribute_position(element.attributes(), None, &name, cancelled)?;
            (position, QualifiedName::new(Namespace::none(), None, name))
        };
        // Discard a normalized lookup spelling before retaining the original
        // existing QName. Set-value preserves its actual prefix and order.
        if let Some(position) = position {
            drop(name);
            drop(name_formation);
            name_formation = style::formation::Formation::new(self, cancelled)?;
            name = retain_name(
                element.attributes().as_slice()[position].name(),
                name_formation.as_ref(),
                cancelled,
            )?;
        }
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
        attribute: &Attribute,
        cause: AttributeCause,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(&self.node(node)?.kind, DomNodeKind::Element(_)) {
            return Err(DomError::WrongNodeKind);
        }
        let name_formation = style::formation::Formation::new(self, cancelled)?;
        let name = retain_name(attribute.name(), name_formation.as_ref(), cancelled)?;
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
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        #[cfg(test)]
        let copy_boundary = attribute_copy_observation::before(
            self,
            &write.name,
            write.name_formation.is_some(),
            cancelled,
        )?;
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let old_attribute = position.map_or(Ok(0), |index| {
            attribute_units(&element.attributes().as_slice()[index])
        })?;
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
        let hook = style::formation::Formation::new(self, cancelled)?;
        let changed = retain_name(&write.name, hook.as_ref(), cancelled)?;
        let details_new_value = retain_string(value, hook.as_ref(), cancelled)?;
        let old_value = position
            .map(|index| {
                retain_string(
                    element.attributes().as_slice()[index].value(),
                    hook.as_ref(),
                    cancelled,
                )
            })
            .transpose()?;
        let candidate = style::formation::Formation::new(self, cancelled)?;
        let replacement = Attribute::new(
            write.name,
            retain_string(value, candidate.as_ref(), cancelled)?,
        );
        let nonce = nonce_after_effect(element, nonce_value, candidate.as_ref(), cancelled)?;
        let mut attributes = Vec::new();
        style::formation::reserve(
            &mut attributes,
            element.attributes().as_slice().len(),
            candidate.as_ref(),
            cancelled,
        )?;
        for (index, attribute) in element.attributes().iter().enumerate() {
            self.source_step(cancelled)?;
            if Some(index) == position {
                continue;
            }
            let copied = Attribute::new(
                retain_name(attribute.name(), candidate.as_ref(), cancelled)?,
                retain_string(attribute.value(), candidate.as_ref(), cancelled)?,
            );
            attributes.push(copied);
        }
        if let Some(position) = position {
            attributes.insert(position, replacement);
        } else {
            style::formation::reserve(&mut attributes, 1, candidate.as_ref(), cancelled)?;
            attributes.push(replacement);
        }
        let attributes = publish_attributes(attributes, candidate.as_ref(), cancelled)?;
        #[cfg(test)]
        attribute_copy_observation::copied(
            copy_boundary,
            attributes.shared_attributes(),
            self,
            cancelled,
        )?;
        let staged = element.with_attributes_formed(
            attributes,
            &changed,
            false,
            nonce,
            candidate.as_ref(),
            cancelled,
        )?;
        self.publish_element(node, old_units, staged, limits)?;
        drop(candidate);
        drop(write.name_formation);
        #[cfg(test)]
        attribute_copy_observation::hooks(
            &changed,
            old_value.as_ref(),
            Some(&details_new_value),
            self,
        );
        let result = self.attribute_hooks(
            node,
            HookChange {
                changed: &changed,
                old_value: old_value.as_ref(),
                new_value: Some(&details_new_value),
                cause,
            },
            limits,
            cancelled,
        );
        drop(old_value);
        drop(details_new_value);
        drop(changed);
        drop(hook);
        #[cfg(test)]
        attribute_copy_observation::released(self);
        result
    }
    fn attribute_hooks(
        &mut self,
        node: Key,
        hook: HookChange<'_>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let HookChange {
            changed,
            old_value,
            new_value,
            cause,
        } = hook;
        let change = if new_value.is_none() {
            AttributeChange::Removed
        } else if old_value.is_some() {
            AttributeChange::Set
        } else {
            AttributeChange::Added
        };
        self.input_attribute_change(node, changed, change, cause, limits, cancelled)?;
        self.script_attribute_change(
            node,
            changed,
            old_value.is_some(),
            new_value.is_none(),
            cancelled,
        )?;
        self.reflected_attribute_change(node, changed);
        self.slot_attribute(node, changed, old_value, limits, cancelled)?;
        self.control_attribute_change(node, changed, old_value, cancelled)?;
        self.select_attribute(node, changed, change, limits, cancelled)?;
        self.details_attribute_change(node, changed, old_value, new_value, limits, cancelled)?;
        self.dialog_attribute_change(
            node,
            changed,
            old_value.is_some(),
            new_value.is_some(),
            cancelled,
        )?;
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
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let name_formation = style::formation::Formation::new(self, cancelled)?;
        let name = if namespace.is_none() && self.html_element_in_html_document(node)? {
            crate::names::ascii_lowercase(
                name,
                &mut || self.source_step(cancelled),
                &mut |input, lower| {
                    style::formation::units(input, lower, name_formation.as_ref(), cancelled)
                },
                &mut |value| retain_string(value, name_formation.as_ref(), cancelled),
            )?
        } else {
            retain_string(name, name_formation.as_ref(), cancelled)?
        };
        let position =
            self.attribute_position(element.attributes(), namespace, &name, cancelled)?;
        drop(name);
        drop(name_formation);
        let Some(position) = position else {
            return Ok(());
        };
        let removed = &element.attributes().as_slice()[position];
        #[cfg(test)]
        let copy_boundary =
            attribute_copy_observation::before(self, removed.name(), false, cancelled)?;
        let hook = style::formation::Formation::new(self, cancelled)?;
        let changed = retain_name(removed.name(), hook.as_ref(), cancelled)?;
        let old_value = retain_string(removed.value(), hook.as_ref(), cancelled)?;
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let candidate = style::formation::Formation::new(self, cancelled)?;
        for _ in 0..surgeist_html::bounded::HTML_NONCE_INSPECTION_BOUND {
            self.source_step(cancelled)?;
        }
        let nonce_value = surgeist_html::nonce_attribute_change(element.name(), &changed, None);
        let nonce = nonce_after_effect(element, nonce_value, candidate.as_ref(), cancelled)?;
        let mut attributes = Vec::new();
        style::formation::reserve(
            &mut attributes,
            element.attributes().as_slice().len(),
            candidate.as_ref(),
            cancelled,
        )?;
        for (index, attribute) in element.attributes().iter().enumerate() {
            self.source_step(cancelled)?;
            if index == position {
                continue;
            }
            attributes.push(Attribute::new(
                retain_name(attribute.name(), candidate.as_ref(), cancelled)?,
                retain_string(attribute.value(), candidate.as_ref(), cancelled)?,
            ));
        }
        let attributes = publish_attributes(attributes, candidate.as_ref(), cancelled)?;
        #[cfg(test)]
        attribute_copy_observation::copied(
            copy_boundary,
            attributes.shared_attributes(),
            self,
            cancelled,
        )?;
        let staged = element.with_attributes_formed(
            attributes,
            &changed,
            true,
            nonce,
            candidate.as_ref(),
            cancelled,
        )?;
        self.publish_element(node, old_units, staged, limits)?;
        drop(candidate);
        #[cfg(test)]
        attribute_copy_observation::hooks(&changed, Some(&old_value), None, self);
        let result = self.attribute_hooks(
            node,
            HookChange {
                changed: &changed,
                old_value: Some(&old_value),
                new_value: None,
                cause: AttributeCause::Change,
            },
            limits,
            cancelled,
        );
        drop(old_value);
        drop(changed);
        drop(hook);
        #[cfg(test)]
        attribute_copy_observation::released(self);
        result
    }
    fn publish_element(
        &mut self,
        node: Key,
        old_units: usize,
        staged: DomElement,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        let units = (self.units - old_units)
            .checked_add(staged.units().ok_or(DomError::RepresentationExhausted)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.node_mut(node).kind = DomNodeKind::Element(staged);
        self.units = units;
        Ok(())
    }
}
fn nonce_after_effect(
    element: &DomElement,
    effect: surgeist_html::HtmlNonceAttributeEffect<'_>,
    budget: Option<&style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<Option<DocumentString>, DomError> {
    use surgeist_html::HtmlNonceAttributeEffect;
    match effect {
        HtmlNonceAttributeEffect::Ignore => element
            .nonce()
            .map(|value| retain_string(value, budget, cancel))
            .transpose(),
        HtmlNonceAttributeEffect::ClearSlot => {
            style::formation::units(&[], false, budget, cancel).map(Some)
        }
        HtmlNonceAttributeEffect::CopyValue(value) => {
            retain_string(value, budget, cancel).map(Some)
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
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(super) fn finish() -> Option<Boundary> {
        ACTIVE.set(false);
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
            for string in [
                name.namespace().uri(),
                name.prefix(),
                Some(name.local_name()),
            ]
            .into_iter()
            .flatten()
            {
                registered_bytes += size_of::<Vec<u16>>()
                    + string.shared_code_units().capacity() * size_of::<u16>();
            }
        }
        Ok(Some(Boundary {
            required: draft.source_storage_bytes(cancel)?
                + draft.source_other_bytes
                + draft.source_formation_bytes.load(Ordering::Relaxed)
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
            // Actual newly owned bank and every retained string occurrence.
            // This is a lower bound: unrelated name/hook/family locals are not included.
            boundary.required +=
                size_of::<Vec<Attribute>>() + attributes.capacity() * size_of::<Attribute>();
            for attr in attributes {
                for string in [
                    attr.name().namespace().uri(),
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
            boundary.admitted = draft.source_storage_bytes(cancel)?
                + draft.source_other_bytes
                + draft.source_formation_bytes.load(Ordering::Relaxed);
            boundary.admitted_units = draft.source_storage_units(cancel)?
                + draft.source_other_units
                + draft.source_temporary_units.load(Ordering::Relaxed);
            LAST.set(Some(boundary));
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
                name.namespace().uri(),
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
    fn actual_attribute_copy_with(
        bytes: usize,
        units: usize,
        append: bool,
        remove: bool,
    ) -> Option<attribute_copy_observation::Boundary> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(16, 2_000_000, 128, 0, 0, 8, 100_000).unwrap()).unwrap();
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
        let mut batch = DomBatch::new();
        if remove {
            batch.remove_attribute(element, "data-31");
        } else {
            batch.set_attribute(element, if append { "data-new" } else { "data-31" }, "new");
        }
        let limits = crate::DomStyleEditLimits::new(bytes, units, 10_000_000, 64, 4096, 4096);
        let Ok(mut lease) = dom.begin_style_batch(batch, limits, &cancel) else {
            return None;
        };
        attribute_copy_observation::start();
        let result = lease.advance(&cancel);
        let observed = attribute_copy_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
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
        let high =
            actual_attribute_copy_with(64_000_000, 10_000_000, false, remove).expect("actual copy");
        let exact = actual_attribute_copy_with(high.admitted, high.admitted_units, false, remove)
            .expect("exact copy envelope is reachable");
        assert!(exact.required <= exact.limit);
        assert!(
            actual_attribute_copy_with(high.admitted - 1, high.admitted_units, false, remove)
                .is_none(),
            "one byte short must stop before copy"
        );
        assert!(
            actual_attribute_copy_with(high.admitted, high.admitted_units - 1, false, remove)
                .is_none(),
            "one UTF16 unit short must stop before copy"
        );
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
    fn source_replacement_copy_has_exact_byte_and_utf16_boundaries() {
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
        let mut dom = Dom::new(DomLimits::new(16, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
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
            crate::DomStyleEditLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096);
        let mut lease = dom.begin_style_batch(batch, limits, &cancel).unwrap();
        assert!(matches!(
            lease.advance(&cancel),
            Ok(crate::DomStyleProgress::Complete(_))
        ));
        drop(lease);
        let current = dom.snapshot();
        let changed = current.node(element).unwrap().as_element().unwrap();
        assert_eq!(
            changed.attributes().as_slice()[0].name().prefix(),
            Some(&"first".into())
        );
        assert_eq!(changed.attributes().as_slice()[0].value(), &value);
        assert_eq!(
            changed.attributes().as_slice()[1].name().local_name(),
            &"tail".into()
        );
        assert_eq!(changed.nonce(), Some(&"new nonce".into()));
        let historical = old.node(element).unwrap().as_element().unwrap();
        assert_eq!(historical.attributes().as_slice()[0].value(), &"old".into());
        assert_eq!(historical.nonce(), Some(&"private".into()));
        let mut remove = DomBatch::new();
        remove.remove_attribute_ns(element, namespace, "item");
        remove.remove_attribute(element, "nonce");
        dom.apply(remove, &cancel).unwrap();
        let removed = dom.snapshot();
        let removed = removed.node(element).unwrap().as_element().unwrap();
        assert_eq!(removed.attributes().as_slice().len(), 1);
        assert_eq!(
            removed.attributes().as_slice()[0].name().local_name(),
            &"tail".into()
        );
        assert_eq!(removed.nonce(), Some(&"".into()));
    }
    #[test]
    fn name_errors_absent_removal_and_late_failure_preserve_atomic_state() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(16, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
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
            crate::DomStyleEditLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096);
        let mut lease = dom.begin_style_batch(late, limits, &cancel).unwrap();
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
