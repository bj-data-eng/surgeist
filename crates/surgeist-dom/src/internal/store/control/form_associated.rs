//! Canonical qualification and mutable state for successful autonomous controls.
use super::*;
use crate::internal::{element::DomElementDefinition, registry::DomRegistryDefinitionId};
use surgeist_html::elements::native::HtmlCustomElementState;

impl Draft {
    pub(in crate::internal::store) fn initialize_form_associated_custom(
        &mut self,
        key: Key,
        definition: DomRegistryDefinitionId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let element = self.node(key)?.element().ok_or(DomError::WrongNodeKind)?;
        let element = element.language();
        if element.form_associated_custom_disabled().is_some()
            || element.custom_element_state() != HtmlCustomElementState::Custom
            || element.custom_element_definition() != DomElementDefinition::Definition(definition)
            || element.has_failed_construction_interface()
        {
            return Ok(());
        }

        let facts = self
            .creation_registry(definition.registry(), cancel)?
            .definition(definition.ordinal())
            .map_err(DomError::SelectCapture)?;
        let mut step = || self.source_step(cancel);
        if !facts.form_associated()
            || !crate::internal::names::in_html_namespace_with(element.name(), &mut step)?
            || !crate::internal::names::same_code_units(
                facts.name().code_units(),
                facts.local_name().code_units(),
                &mut step,
            )?
            || !crate::internal::names::same_code_units(
                facts.local_name().code_units(),
                element.name().local_name().code_units(),
                &mut step,
            )?
        {
            return Ok(());
        }

        let mut fields = self.element_fields_mut(key, cancel)?;
        fields.set_form_associated_custom_disabled(false);
        fields.commit(cancel)?;
        self.reset_control(key, cancel)?;
        self.refresh_form_associated_custom_disabled(key, cancel)
    }

    pub(in crate::internal::store) fn form_associated_custom_disabled(
        &self,
        key: Key,
    ) -> Result<Option<bool>, DomError> {
        Ok(self
            .node(key)?
            .element()
            .and_then(|element| element.language().form_associated_custom_disabled()))
    }

    pub(in crate::internal::store) fn set_form_associated_custom_disabled(
        &mut self,
        key: Key,
        disabled: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let before = self
            .form_associated_custom_disabled(key)?
            .ok_or(DomError::WrongNodeKind)?;
        if before == disabled {
            return Ok(());
        }
        let mut fields = self.element_fields_mut(key, cancel)?;
        fields.set_form_associated_custom_disabled(disabled);
        fields.commit(cancel)?;
        self.record_form_associated_disabled_change(key, disabled, cancel)
    }
}

/// Original ordered native facts retained until publication of their Source segment.
#[derive(Clone, Copy)]
pub(in crate::internal::store) enum Change {
    Owner { element: Key, owner: Option<Key> },
    Disabled { element: Key, disabled: bool },
}

impl Draft {
    pub(in crate::internal::store) fn record_form_associated_owner_change(
        &mut self,
        element: Key,
        old: Option<Key>,
        owner: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if old.map(|id| self.live_key(id)) == owner.map(|id| self.live_key(id)) {
            return Ok(());
        }
        self.record_form_associated_custom_change(Change::Owner { element, owner }, cancel)
    }

    fn record_form_associated_disabled_change(
        &mut self,
        element: Key,
        disabled: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.record_form_associated_custom_change(Change::Disabled { element, disabled }, cancel)
    }

    fn record_form_associated_custom_change(
        &mut self,
        change: Change,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.source_limits.is_none() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let mut changes = self
            .form_associated_custom_changes
            .take()
            .unwrap_or_else(|| self.effect_vec());
        let result = self.select_reserve(&mut changes, 1, cancel);
        if result.is_ok() {
            changes.push(change);
        }
        self.form_associated_custom_changes = Some(changes);
        result
    }

    pub(in crate::internal::store) fn append_form_associated_custom_changes(
        &self,
        ids: &source::AllocationIds,
        output: &mut Vec<DomChange>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(pending) = &self.form_associated_custom_changes else {
            return Ok(());
        };
        source::formation::reserve(output, pending.len(), budget, cancel)?;
        let resolve = |key| match key {
            Key::Existing(id) => Ok(id),
            Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation),
        };
        for change in pending.iter() {
            self.source_step(cancel)?;
            output.push(match *change {
                Change::Owner { element, owner } => DomChange::FormAssociatedCustomElementOwner {
                    element: resolve(element)?,
                    owner: owner.map(resolve).transpose()?,
                },
                Change::Disabled { element, disabled } => {
                    DomChange::FormAssociatedCustomElementDisabled {
                        element: resolve(element)?,
                        disabled,
                    }
                }
            });
        }
        Ok(())
    }
}
