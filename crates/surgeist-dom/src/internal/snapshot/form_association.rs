//! Qualified association facts from one canonical snapshot.

use super::{DomElement, DomError, DomNode, NodeId};
use crate::internal::{
    element::DomElementDefinition,
    names::{in_html_namespace_with, same_code_units},
};
use surgeist_html::{attributes::HtmlInputType, elements::form::HtmlNativeFormAssociationKind};

pub(super) enum CustomFormAssociation {
    Initialized,
    Defined,
}

enum ListedControl {
    Native(HtmlNativeFormAssociationKind),
    Custom(CustomFormAssociation),
}

impl DomNode<'_> {
    /// Establish an autonomous custom control from its actual state or definition.
    /// The caller has already qualified the Element's HTML namespace.
    pub(super) fn custom_form_association_with<E: From<DomError>>(
        self,
        element: &DomElement,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<CustomFormAssociation>, E> {
        step()?;
        if element.form_associated_custom_disabled().is_some() {
            return Ok(Some(CustomFormAssociation::Initialized));
        }
        let DomElementDefinition::Definition(definition) = element.custom_element_definition()
        else {
            return Ok(None);
        };
        let registry = self
            .snapshot
            .state
            .registries
            .as_ref()
            .ok_or(DomError::StaleNode)?
            .get_with(definition.registry(), &mut *step)?;
        step()?;
        let facts = registry
            .definition(definition.ordinal())
            .map_err(|_| DomError::StaleNode)?;
        if facts.form_associated()
            && same_code_units(
                facts.name().code_units(),
                facts.local_name().code_units(),
                &mut *step,
            )?
            && same_code_units(
                facts.local_name().code_units(),
                element.name().local_name().code_units(),
                step,
            )?
        {
            return Ok(Some(CustomFormAssociation::Defined));
        }
        Ok(None)
    }

    fn listed_control_with<E: From<DomError>>(
        self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<ListedControl>, E> {
        step()?;
        let Some(element) = self.as_element() else {
            return Ok(None);
        };
        if let Some((kind, _)) = element.native_listed_form_association() {
            return Ok(Some(ListedControl::Native(kind)));
        }
        if !in_html_namespace_with(element.name(), &mut *step)? {
            return Ok(None);
        }
        Ok(self
            .custom_form_association_with(element, step)?
            .map(ListedControl::Custom))
    }

    pub(crate) fn is_listed_control_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        Ok(self.listed_control_with(&mut step)?.is_some())
    }

    pub(crate) fn is_listed_by_form_with<E: From<DomError>>(
        self,
        form: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        let owner = self.node.control_owner();
        if owner.is_some_and(|owner| owner != form) {
            return Ok(false);
        }
        match self.listed_control_with(&mut step)? {
            Some(ListedControl::Native(HtmlNativeFormAssociationKind::Input))
                if owner.is_some() =>
            {
                Ok(self
                    .as_input()
                    .ok_or(DomError::WrongNodeKind)?
                    .input_type_with(step)?
                    != HtmlInputType::Image)
            }
            Some(ListedControl::Custom(CustomFormAssociation::Defined)) => {
                Err(DomError::IncompleteSourceOperation.into())
            }
            Some(_) => Ok(owner.is_some()),
            None => Ok(false),
        }
    }
}
