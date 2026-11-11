//! Insertion and removal of nodes in the active session storage.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InsertionCause {
    Parser,
    NonParser,
}

impl SessionStorage {
    /// Prepared child order. The real replacement vector is admitted and every
    /// entry is read/copied before use. Commit later consists only of vector swaps.
    pub(super) fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        cause: InsertionCause,
    ) -> Result<(), SessionError> {
        self.insert_with_meta_csp(control, parent, node, before, cause, &mut None)
    }
    pub(super) fn insert_with_meta_csp(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        cause: InsertionCause,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let parent = self.index(control, parent)?;
        let node = self.index(control, node)?;
        let before = before.map(|node| self.index(control, node)).transpose()?;
        self.insert_indices_with_cause(control, parent, node, before, cause)?;
        self.drain_construction_effects_with_meta_csp(control, service)
    }

    pub(super) fn insert_indices(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
        node: usize,
        before: Option<usize>,
    ) -> Result<(), SessionError> {
        self.insert_indices_with_cause(control, parent, node, before, InsertionCause::NonParser)
    }
    pub(super) fn insert_indices_with_cause(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
        node: usize,
        before: Option<usize>,
        cause: InsertionCause,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        self.container(control, parent)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .check_detachment()
            .map_err(|error| construction_error(control, error))?;
        if let Some(before) = before {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            self.construction
                .node_at(before)
                .ok_or(SessionError::ForeignNode)?
                .check_reference_child_of(
                    self.construction
                        .node_at(parent)
                        .ok_or(SessionError::ForeignNode)?,
                )
                .map_err(|error| construction_error(control, error))?;
            if before == node {
                // Pre-insert adjusts the reference child but still adopts
                // (removes) and inserts the root. Final links/order are equal;
                // actual managed image facts nevertheless receive both hooks.
                if let Some(mut effects) =
                    self.prepare_form_effects(control, node, true, true, Some(parent))?
                {
                    payload_write::<InsertionCause>(control, 1)?;
                    effects.cause = cause;
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                }
                self.nonce_inserted_tree(control, node)?;
                return Ok(());
            }
        }
        let old_parent = self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .map(|node| node.position());
        let mut form_effects =
            self.prepare_form_effects(control, node, old_parent.is_some(), true, Some(parent))?;
        if let Some(effects) = &mut form_effects {
            payload_write::<InsertionCause>(control, 1)?;
            effects.cause = cause;
        }
        let reroot = self.begin_reroot(control, node, Some(parent))?;
        control.charge(
            HtmlParseWorkCategory::ProviderCall,
            if before.is_some() { 3 } else { 2 },
        )?;
        let node_handle = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let parent_handle = self
            .construction
            .node_at(parent)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let before_handle = before
            .map(|position| {
                self.construction
                    .node_at(position)
                    .map(|record| record.handle())
                    .ok_or_else(|| latch_registry_row(control))
            })
            .transpose()?;
        let requests = &[(
            HtmlParseResource::ManagedEdges,
            u64::from(old_parent.is_none()),
        )];
        let result = self.insert_record(
            control,
            &node_handle,
            (parent, &parent_handle),
            before_handle.as_ref(),
            form_effects.is_some(),
            requests,
        );
        self.end_reroot(reroot);
        let revoke_standalone = result?;
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        if revoke_standalone {
            self.standalone_ineligibility = Some(HtmlParseUnsupportedState::ScriptDocument);
        }
        self.nonce_inserted_tree(control, node)?;
        Ok(())
    }

    pub(super) fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        let node = self.index(control, node)?;
        self.remove_index(control, node)?;
        self.drain_construction_effects(control)
    }

    pub(super) fn remove_index(
        &mut self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .check_detachment()
            .map_err(|error| construction_error(control, error))?;
        if self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .is_none()
        {
            return Ok(());
        }
        let form_effects = self.prepare_form_effects(control, node, true, false, None)?;
        if form_effects.is_some() {
            payload_write::<Option<FormEffects>>(control, 1)?;
        }
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let handle = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let reroot = self.begin_reroot(control, node, None)?;
        let detached = self.detach_record(control, &handle);
        self.end_reroot(reroot);
        detached?;
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        Ok(())
    }
}
