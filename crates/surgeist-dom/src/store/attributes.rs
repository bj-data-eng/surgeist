use super::*;
use crate::names::{attribute_name, name_units, qualified_equals, valid_attribute_local_name};
use crate::{Attribute, Namespace, OrderedAttributes};
use surgeist_html::bounded::{HtmlAttributeStorage, HtmlAttributeValidation};

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
        let old_units = position.map_or(Ok(0), |position| {
            attribute_units(&element.attributes().as_slice()[position])
        })?;
        let units = (self.units - old_units)
            .checked_add(attribute_units(&replacement)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let mut attributes = copy(element.attributes().as_slice())?;
        if let Some(position) = position {
            attributes[position] = replacement;
        } else {
            reserve(&mut attributes, 1)?;
            attributes.push(replacement);
        }
        let attributes = publish_attributes(attributes, cancelled)?;
        let DomNodeKind::Element(element) = &mut self.node_mut(node).kind else {
            unreachable!("checked Element")
        };
        element.set_attributes(attributes);
        self.units = units;
        Ok(())
    }

    pub(super) fn remove_attribute(
        &mut self,
        node: Key,
        namespace: Option<Namespace>,
        name: DocumentString,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            return Err(DomError::WrongNodeKind);
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
        let units = self.units - attribute_units(&element.attributes().as_slice()[position])?;
        let mut attributes = copy(element.attributes().as_slice())?;
        attributes.remove(position);
        let attributes = publish_attributes(attributes, cancelled)?;
        let DomNodeKind::Element(element) = &mut self.node_mut(node).kind else {
            unreachable!("checked Element")
        };
        element.set_attributes(attributes);
        self.units = units;
        Ok(())
    }
}
