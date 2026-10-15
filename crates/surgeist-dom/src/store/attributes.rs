use super::*;
use crate::names::{attribute_name, name_units, qualified_equals, valid_attribute_local_name};
use crate::{Attribute, Namespace, OrderedAttributes};
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

fn attribute_units(attribute: &Attribute) -> Result<usize, DomError> {
    name_units(attribute.name())
        .and_then(|units| units.checked_add(attribute.value().len()))
        .ok_or(DomError::RepresentationExhausted)
}

fn publish_attributes(
    attributes: Vec<Attribute>,
    cancelled: &AtomicBool,
) -> Result<OrderedAttributes, DomError> {
    let mut validation =
        HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes)).validate();
    while let Some(action) = validation.next_action().expect("unique expanded names") {
        check_cancel(cancelled)?;
        action
            .advance()
            .expect("source attribute identity preserved");
    }
    let validated = HtmlAttributeValidation::try_finish(&mut Some(validation))
        .expect("completed unique attributes");
    Ok(validated.into_attributes())
}

impl Draft {
    pub(super) fn set_nonce(
        &mut self,
        node: Key,
        value: DocumentString,
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
        let DomNodeKind::Element(element) = &mut self.node_mut(node).kind else {
            unreachable!("checked Element")
        };
        element.set_nonce(value);
        self.units = units;
        Ok(())
    }
    pub(super) fn set_attribute(
        &mut self,
        node: Key,
        namespace: Option<Namespace>,
        name: DocumentString,
        value: DocumentString,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let (position, name) = if let Some(namespace) = namespace {
            let name = attribute_name(namespace, name, cancelled)?;
            let position = element.attributes().iter().position(|attribute| {
                attribute.name().namespace() == name.namespace()
                    && attribute.name().local_name() == name.local_name()
            });
            (position, name)
        } else {
            if !valid_attribute_local_name(&name, cancelled)? {
                return Err(DomError::InvalidCharacter);
            }
            let name = if self.html_element_in_html_document(node)? {
                crate::names::ascii_lowercase(name, cancelled)?
            } else {
                name
            };
            let position = element
                .attributes()
                .iter()
                .position(|attribute| qualified_equals(attribute.name(), &name));
            (
                position,
                crate::QualifiedName::new(Namespace::none(), None, name),
            )
        };
        // Source set-value preserves an existing attribute's name and prefix.
        let name = position.map_or(name, |position| {
            element.attributes().as_slice()[position].name().clone()
        });
        let replacement = Attribute::new(name, value);
        self.write_attribute(
            node,
            position,
            replacement,
            AttributeCause::Change,
            limits,
            cancelled,
        )
    }

    /// Append exact records checked by the native component entry or canonical
    /// clone source. Do not reconstruct an NS operand: literal unnamespaced
    /// xml:lang and actual supplied prefix/local fields stay distinct.
    pub(super) fn append_attribute(
        &mut self,
        node: Key,
        attribute: Attribute,
        cause: AttributeCause,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.write_attribute(node, None, attribute, cause, limits, cancelled)
    }

    fn write_attribute(
        &mut self,
        node: Key,
        position: Option<usize>,
        replacement: Attribute,
        cause: AttributeCause,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancelled)?;
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let nonce = nonce_after_attribute(element, replacement.name(), Some(replacement.value()));
        let old_attribute = position.map_or(Ok(0), |index| {
            attribute_units(&element.attributes().as_slice()[index])
        })?;
        // Admit raw caller storage before numerical scans/materialization. The
        // old derived digits are excluded: a shrinking replacement may remove
        // them. Final admission below counts the complete candidate exactly.
        let base = self.units
            - old_attribute
            - element.nonce().map_or(0, DocumentString::len)
            - element
                .family_units()
                .ok_or(DomError::RepresentationExhausted)?;
        let minimum = base
            .checked_add(attribute_units(&replacement)?)
            .and_then(|units| units.checked_add(nonce.as_ref().map_or(0, DocumentString::len)))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(minimum, limits.units, DomResource::StringCodeUnits)?;
        let changed = replacement.name().clone();
        let old_value =
            position.map(|position| element.attributes().as_slice()[position].value().clone());
        let mut staged = element.clone();
        let mut attributes = copy(element.attributes().as_slice())?;
        if let Some(position) = position {
            attributes[position] = replacement;
        } else {
            reserve(&mut attributes, 1)?;
            attributes.push(replacement);
        }
        let attributes = publish_attributes(attributes, cancelled)?;
        staged.apply_attributes(attributes, &changed, false, cancelled)?;
        if let Some(nonce) = nonce {
            staged.set_nonce(nonce);
        }
        self.publish_element(node, old_units, staged, limits)?;
        self.input_attribute_change(
            node,
            &changed,
            if old_value.is_some() {
                AttributeChange::Set
            } else {
                AttributeChange::Added
            },
            cause,
            limits,
            cancelled,
        )?;
        self.script_attribute_change(node, &changed, old_value.is_some(), false, cancelled)?;
        self.reflected_attribute_change(node, &changed);
        self.slot_attribute(node, &changed, old_value.clone(), limits, cancelled)?;
        self.control_attribute_change(node, &changed, old_value.as_ref(), cancelled)
    }

    pub(super) fn remove_attribute(
        &mut self,
        node: Key,
        namespace: Option<Namespace>,
        name: DocumentString,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let name = if namespace.is_none() && self.html_element_in_html_document(node)? {
            crate::names::ascii_lowercase(name, cancelled)?
        } else {
            name
        };
        let position = element.attributes().iter().position(|attribute| {
            namespace.as_ref().map_or_else(
                || qualified_equals(attribute.name(), &name),
                |namespace| {
                    attribute.name().namespace() == namespace
                        && attribute.name().local_name() == &name
                },
            )
        });
        let Some(position) = position else {
            return Ok(());
        };
        let removed = &element.attributes().as_slice()[position];
        let changed = removed.name().clone();
        let old_value = Some(removed.value().clone());
        let nonce = nonce_after_attribute(element, removed.name(), None);
        let old_units = element.units().ok_or(DomError::RepresentationExhausted)?;
        let mut staged = element.clone();
        let mut attributes = copy(element.attributes().as_slice())?;
        attributes.remove(position);
        let attributes = publish_attributes(attributes, cancelled)?;
        staged.apply_attributes(attributes, &changed, true, cancelled)?;
        if let Some(nonce) = nonce {
            staged.set_nonce(nonce);
        }
        self.publish_element(node, old_units, staged, limits)?;
        self.input_attribute_change(
            node,
            &changed,
            AttributeChange::Removed,
            AttributeCause::Change,
            limits,
            cancelled,
        )?;
        self.script_attribute_change(node, &changed, true, true, cancelled)?;
        self.reflected_attribute_change(node, &changed);
        self.slot_attribute(node, &changed, old_value.clone(), limits, cancelled)?;
        self.control_attribute_change(node, &changed, old_value.as_ref(), cancelled)
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

fn nonce_after_attribute(
    element: &DomElement,
    attribute: &crate::QualifiedName,
    value: Option<&DocumentString>,
) -> Option<DocumentString> {
    use surgeist_html::HtmlNonceAttributeEffect;
    match surgeist_html::nonce_attribute_change(element.name(), attribute, value) {
        HtmlNonceAttributeEffect::Ignore => element.nonce().cloned(),
        HtmlNonceAttributeEffect::ClearSlot => Some("".into()),
        HtmlNonceAttributeEffect::CopyValue(value) => Some(value.clone()),
    }
}
