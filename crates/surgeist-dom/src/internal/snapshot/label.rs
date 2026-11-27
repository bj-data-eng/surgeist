//! HTML Label association over the ordinary tree in an admitted snapshot.

use super::{
    DescendantSearch, DomError, DomNode, NodeId, ReservedDescendantSearch,
    element_id::NonemptyElementId,
};
use crate::internal::{
    element::DomElementDefinition,
    form::DomForm,
    names::{in_html_namespace_with, same_code_units},
};
use surgeist_html::{
    elements::form::{HtmlNativeFormAssociationKind, native_labelability_with},
    vocabulary::KnownHtmlElement,
};

const FOR: &[u16] = &[0x66, 0x6f, 0x72];

pub(crate) enum LabeledControl<'a> {
    Native(DomNode<'a>, KnownHtmlElement),
    Custom(DomNode<'a>),
    UnsettledCustom(DomNode<'a>),
}

#[derive(Clone, Copy)]
pub(crate) enum LabelTarget<'a> {
    Explicit(NonemptyElementId<'a>),
    Implicit,
}

impl<'a> LabelTarget<'a> {
    /// Resolve this target through the native Label association rules.
    /// Reverse lookups may require one exact control and avoid qualifying
    /// an explicit ID match that already proves a different identity.
    pub(crate) fn resolve_with<E: From<DomError>>(
        self,
        label: DomNode<'a>,
        expected: Option<NodeId>,
        mut step: impl FnMut() -> Result<(), E>,
        search: &mut impl DescendantSearch<'a, E>,
    ) -> Result<Option<LabeledControl<'a>>, E> {
        match self {
            Self::Explicit(id) => {
                let root = label.root_with(false, &mut step)?;
                let Some(first) = root.resolve_element_id_with(id, true, &mut step, search)? else {
                    return Ok(None);
                };
                if expected.is_some_and(|expected| expected != first.id()) {
                    return Ok(None);
                }
                first.as_labeled_control(&mut step)
            }
            Self::Implicit => {
                let first = search.find(label, step, |node, step| node.as_labeled_control(step))?;
                Ok(
                    first
                        .filter(|control| expected.is_none_or(|expected| expected == control.id())),
                )
            }
        }
    }
}

impl LabeledControl<'_> {
    fn id(&self) -> NodeId {
        match self {
            Self::Native(node, _) | Self::Custom(node) | Self::UnsettledCustom(node) => node.id(),
        }
    }
}

impl<'a> DomNode<'a> {
    /// Read the labeled control of this HTML Label in its ordinary tree.
    ///
    /// A present `for` uses the first exact DOM ID. An absent `for` uses the
    /// first labelable ordinary descendant. A live caller qualifies access to
    /// the current published Source segment.
    ///
    /// `step` admits the reached reads and comparisons. `reserve` admits each
    /// new traversal allocation while its predecessor remains retained.
    ///
    /// # Errors
    ///
    /// Returns the first admission or lookup failure, or
    /// [`DomError::WrongNodeKind`] for a receiver outside the HTML Label interface.
    pub fn html_labeled_control_with<E: From<DomError>, Backing>(
        self,
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<NodeId>, E> {
        Ok(self
            .labeled_control_with(step, reserve)?
            .map(|control| control.id()))
    }

    /// Read the stored Form owner of this HTML Label's current labeled control.
    ///
    /// Uses the same reached tree and admission as [`Self::html_labeled_control_with`].
    /// The selected control and Form identity belong to this snapshot.
    ///
    /// # Errors
    ///
    /// Returns the first admission or lookup failure, [`DomError::WrongNodeKind`]
    /// for an invalid receiver or stored Form, or [`DomError::IncompleteSourceOperation`]
    /// if the selected form-associated custom element's owner is still unsettled.
    pub fn html_label_form_owner_with<E: From<DomError>, Backing>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<DomForm<'a>>, E> {
        match self.labeled_control_with(&mut step, reserve)? {
            Some(LabeledControl::Custom(node)) => node.stored_form_owner_with(step),
            Some(LabeledControl::Native(node, kind))
                if HtmlNativeFormAssociationKind::from_element(kind).is_some() =>
            {
                node.stored_form_owner_with(step)
            }
            Some(LabeledControl::UnsettledCustom(_)) => {
                Err(DomError::IncompleteSourceOperation.into())
            }
            _ => Ok(None),
        }
    }

    fn labeled_control_with<E: From<DomError>, Backing>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<LabeledControl<'a>>, E> {
        if !self.is_html_label_with(&mut step)? {
            return Err(DomError::WrongNodeKind.into());
        }
        let Some(target) = self.label_target_with(&mut step)? else {
            return Ok(None);
        };
        target.resolve_with(self, None, step, &mut ReservedDescendantSearch(reserve))
    }

    pub(crate) fn is_html_label_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        let Some(element) = self.as_element() else {
            return Ok(false);
        };
        let name = element.name();
        if !in_html_namespace_with(name, &mut step)? {
            return Ok(false);
        }
        step()?;
        Ok(KnownHtmlElement::from_units(name.local_name().code_units())
            == Some(KnownHtmlElement::Label))
    }

    pub(crate) fn label_target_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<LabelTarget<'a>>, E> {
        let attributes = self.attributes().ok_or(DomError::WrongNodeKind)?;
        match attributes.get_null_namespace_attribute_with(FOR, step)? {
            Some(attribute) => {
                Ok(NonemptyElementId::new(attribute.value().code_units())
                    .map(LabelTarget::Explicit))
            }
            None => Ok(Some(LabelTarget::Implicit)),
        }
    }

    pub(crate) fn native_label_list_applicable_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        let element = self.as_element().ok_or(DomError::WrongNodeKind)?;
        let name = element.name();
        if !in_html_namespace_with(name, &mut step)? {
            return Err(DomError::WrongNodeKind.into());
        }
        step()?;
        let kind = KnownHtmlElement::from_units(name.local_name().code_units())
            .ok_or(DomError::WrongNodeKind)?;
        let labelable = self.native_labelable_with(kind, &mut step)?;
        if !labelable && kind != KnownHtmlElement::Input {
            return Err(DomError::WrongNodeKind.into());
        }
        Ok(labelable)
    }

    fn native_labelable_with<E: From<DomError>>(
        self,
        kind: KnownHtmlElement,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        native_labelability_with(kind, || {
            self.as_input()
                .ok_or(DomError::WrongNodeKind)?
                .input_type_with(step)
        })
    }

    fn as_labeled_control<E: From<DomError>>(
        self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<LabeledControl<'a>>, E> {
        step()?;
        let Some(element) = self.as_element() else {
            return Ok(None);
        };
        let name = element.name();
        if !in_html_namespace_with(name, &mut *step)? {
            return Ok(None);
        }
        step()?;
        let local = name.local_name().code_units();
        if let Some(kind) = KnownHtmlElement::from_units(local) {
            let labelable = self.native_labelable_with(kind, step)?;
            return Ok(labelable.then_some(LabeledControl::Native(self, kind)));
        }

        step()?;
        if element.form_associated_custom_disabled().is_some() {
            return Ok(Some(LabeledControl::Custom(self)));
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
            && same_code_units(facts.local_name().code_units(), local, step)?
        {
            return Ok(Some(LabeledControl::UnsettledCustom(self)));
        }
        Ok(None)
    }
}
