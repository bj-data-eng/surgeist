//! The parser session facade: one arena at a time, its tree operations, freezing and release.

use super::*;
use surgeist_html::{
    construction::HtmlConstructionResult,
    construction::{HtmlFragmentOwner, HtmlNodeKind, HtmlTemplateContentsKind},
    values::QualifiedName,
};

pub(crate) struct ParserArenaSession {
    pub(super) storage: Option<SessionStorage>,
}

/// Borrowed transitions retain the owner on every failure, including terminal
/// stops. Storage is absent only after successful freeze or explicit cleanup.
impl ParserArenaSession {
    pub(crate) fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        Ok(Self {
            storage: Some(SessionStorage::new(control, root)?),
        })
    }
    pub(super) fn active(&self) -> Result<&SessionStorage, SessionError> {
        self.storage.as_ref().ok_or(SessionError::Consumed)
    }
    pub(super) fn active_mut(&mut self) -> Result<&mut SessionStorage, SessionError> {
        self.storage.as_mut().ok_or(SessionError::Consumed)
    }
    pub(crate) fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.active()?.root(control)
    }
    pub(crate) fn fragment_root(
        &self,
        control: &mut ParseControl,
    ) -> Result<Option<ParserNode>, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Fragment
        ) {
            Ok(Some(storage.root(control)?))
        } else {
            Ok(None)
        }
    }
    pub(crate) fn validate_node(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        self.active()?.index(control, node)?;
        Ok(())
    }
    /// The validated arena position that is the node's identity: two nodes are
    /// the same node exactly when their positions are equal.
    pub(crate) fn node_position(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<usize, SessionError> {
        self.active()?.index(control, node)
    }
    pub(crate) fn same_node(
        &self,
        control: &mut ParseControl,
        left: &ParserNode,
        right: &ParserNode,
    ) -> Result<bool, SessionError> {
        Ok(self.active()?.index(control, left)? == self.active()?.index(control, right)?)
    }
    pub(crate) fn parent(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        storage
            .construction
            .node_at(index)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .map(|node| storage.node(control, node.position()))
            .transpose()
    }
    pub(crate) fn element_name<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<&'a QualifiedName>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(
            match &storage
                .construction
                .node_at(index)
                .ok_or_else(|| latch_registry_row(control))?
                .kind()
            {
                HtmlNodeKind::Element(value) => Some(value.name()),
                _ => None,
            },
        )
    }
    /// The facts required by HTML's insert-an-element operation (§13.2.6.1).
    /// This observes the existing arena, without allocating or moving a node.
    /// Association ancestry comes only from an actual template/shadow owner;
    /// an associated fragment's absent ordinary parent is not its ancestry.
    pub(crate) fn element_insertion_ignored(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
        target_parent: &ParserNode,
    ) -> Result<Option<ElementInsertionIgnored>, SessionError> {
        let storage = self.active()?;
        let element = storage.index(control, element)?;
        let target = storage.index(control, target_parent)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let record = storage
            .construction
            .node_at(element)
            .ok_or(SessionError::ForeignNode)?;
        if !matches!(record.kind(), HtmlNodeKind::Element(_)) {
            return Err(SessionError::WrongToken);
        }
        if record.parent().is_some() {
            return Ok(Some(ElementInsertionIgnored::ParentAlreadyPresent));
        }
        // An element with no children, no shadow root and, if it owns template
        // contents, empty and shadowless ones has nothing below it but those
        // contents. It is then a host-including inclusive ancestor of the
        // target only as the target itself or as the owner of the target that
        // is its contents. Every element the parser inserts is such an element.
        let contents = match record.template_contents_kind() {
            Some(HtmlTemplateContentsKind::Owned) => record.template_contents(),
            _ => None,
        };
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let shallow = record.child_count() == 0
            && record.shadow_root().is_none()
            && contents.is_none_or(|contents| {
                contents.child_count() == 0 && contents.shadow_root().is_none()
            });
        if shallow {
            if target == element || contents.is_some_and(|contents| contents.position() == target) {
                return Ok(Some(
                    ElementInsertionIgnored::HostIncludingInclusiveAncestor,
                ));
            }
        } else {
            let mut ancestor = Some(target);
            for _ in 0..storage.construction.storage(control)?.len() {
                let Some(index) = ancestor else { break };
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if index == element {
                    return Ok(Some(
                        ElementInsertionIgnored::HostIncludingInclusiveAncestor,
                    ));
                }
                let record = storage
                    .construction
                    .node_at(index)
                    .ok_or(SessionError::ForeignNode)?;
                ancestor = if let Some(owner) = &record.fragment_owner() {
                    let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
                        owner;
                    control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                    let owner_position = storage
                        .construction
                        .storage(control)?
                        .node_by_result_id(owner)
                        .map_err(|_| SessionError::ForeignNode)?
                        .position();
                    Some(owner_position)
                } else {
                    record.parent().map(|node| node.position())
                };
            }
            if ancestor.is_some() {
                return Err(SessionError::Cycle);
            }
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if matches!(
            storage
                .construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            for position in 0..storage
                .construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?
                .child_count()
            {
                // One actual child-index read and one referenced kind read.
                control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                let child = storage
                    .construction
                    .node_at(target)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_at(position)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                let record = storage
                    .construction
                    .node_at(child)
                    .ok_or(SessionError::ForeignNode)?;
                if matches!(record.kind(), HtmlNodeKind::Element(_)) {
                    return Ok(Some(ElementInsertionIgnored::DocumentAlreadyHasElement));
                }
            }
        }
        Ok(None)
    }
    /// Only the actual token-created template path in this checkpoint proves
    /// its insertion target and end marker are initially null. Preexisting
    /// templates carry no such parser creation provenance.
    #[cfg(test)]
    pub(crate) fn unpatched_template_contents(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        let contents = storage.template_contents(control, node)?;
        if contents.is_none() {
            return Ok(None);
        }
        for payload in &storage.payloads.values {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::String { location: StringLocation::ElementName(owner), .. } if *owner == index)
            {
                return Ok(contents);
            }
        }
        Err(SessionError::TemplateInsertionStateUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.active()?.template_contents(control, element)
    }
    pub(crate) fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        self.qualify_intrinsic_mutation(control, node, capability)?;
        self.active_mut()?
            .insert(control, parent, node, before, InsertionCause::NonParser)
    }
    /// Actual parser insertion; qualification precedes topology/effect mutation.
    #[cfg(test)]
    pub(crate) fn insert_parser(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        self.insert_parser_with_meta_csp(control, parent, node, before, capability, &mut None)
    }
    pub(crate) fn insert_parser_with_meta_csp(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        capability.validate_session(control, self)?;
        self.qualify_base_response(control, node, capability)?;
        self.qualify_intrinsic_mutation(control, node, Some(capability))?;
        self.active_mut()?.insert_with_meta_csp(
            control,
            parent,
            node,
            before,
            InsertionCause::Parser,
            service,
        )
    }
    pub(crate) fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        self.qualify_intrinsic_mutation(control, node, capability)?;
        self.active_mut()?.remove(control, node)
    }
    fn qualify_intrinsic_mutation(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        let root = storage.index(c, node)?;
        let mut next = Some(root);
        while let Some(index) = next {
            if storage.svg_script_local(c, index)? {
                storage.svg_state_at(c, index)?;
                let capability = capability.ok_or_else(|| {
                    SessionError::Stop(
                        c.stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance),
                    )
                })?;
                capability.validate_session(c, self)?;
            }
            for name in ["select", "option", "selectedcontent", "script"] {
                if storage.html_local(c, index, name)? {
                    if name == "script"
                        && script_metadata(&storage.payloads, c, index)?
                            .1
                            .parser_document
                            .is_some()
                    {
                        continue;
                    }
                    let capability = capability.ok_or_else(|| {
                        SessionError::Stop(
                            c.stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance),
                        )
                    })?;
                    capability.validate_session(c, self)?;
                    return Ok(());
                }
            }
            next = storage.next_effect_node(c, root, index)?;
        }
        Ok(())
    }
    /// Called after a real parser stack removal, never during terminal cleanup.
    pub(crate) fn parser_stack_removed(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        self.qualify_stack_removal(control, node, capability)?;
        let storage = self.active_mut()?;
        let node = storage.index(control, node)?;
        storage.style_stack_changed(control, node, false)?;
        if storage.html_local(control, node, "option")? {
            storage.update_for_option(control, node)?;
            return storage.drain_construction_effects(control);
        }
        if !storage.html_local(control, node, "textarea")? {
            return Ok(());
        }
        match storage.textarea_slot(control, node) {
            Ok(Some(_)) => {}
            // No state row, or a row of another kind, for a textarea.
            Ok(None) | Err(SessionError::WrongToken | SessionError::ForeignNode) => {
                return Err(latch_registry_row(control));
            }
            Err(error) => return Err(error),
        }
        match storage.refresh_textarea(control, node, None, TextareaRefresh::Reset) {
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(error) => Err(latch_effect_fault(control, error)),
            Ok(()) => Ok(()),
        }
    }
    pub(crate) fn qualify_stack_removal(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        if storage.html_local(c, index, "option")? {
            let capability = capability.ok_or_else(|| {
                SessionError::Stop(
                    c.stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance),
                )
            })?;
            capability.validate_session(c, self)?;
            storage.select_metadata(c, index)?;
        }
        Ok(())
    }
    pub(crate) fn freeze(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<FrozenArena, SessionError> {
        let fragment_root = self.active_mut()?.prepare_freeze(control)?;
        self.active_mut()?.prepare_association_documents(control)?;
        // A stopped append keeps its buffer for cleanup and cannot publish.
        // The check precedes publication, so a refusal consumes nothing.
        if self.active()?.failed_text_append.is_some() {
            return Err(latch_registry_row(control));
        }
        // Only a parser-initialized session carries this actual variable owner.
        // Prepay its active-to-frozen move before storage is taken; a stopped
        // share restores that same owner without useful recovery work.
        if self.active()?.evidence.is_some() {
            payload_write::<Option<ParseEvidence>>(control, 2)?;
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let root_position = fragment_root.unwrap_or(0);
        let root = self
            .active()?
            .construction
            .node_at(root_position)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let (result, lease) = self.active_mut()?.construction.publish(control, root)?;
        let SessionStorage {
            csp_slot: _,
            control: owner,
            effects,
            evidence,
            fragment,
            #[cfg(test)]
                cancel_after_details_shift: _,
            #[cfg(test)]
                cancel_after_radio_peer: _,
            #[cfg(test)]
                cancel_after_removal: _,
            #[cfg(test)]
                cancel_after_textarea_reset: _,
            #[cfg(test)]
                cancel_after_output_reset: _,
            #[cfg(test)]
                cancel_after_input_reset: _,
            #[cfg(test)]
                lose_association_row: _,
            #[cfg(test)]
                cancel_after_range_append: _,
            #[cfg(test)]
                cancel_after_range_ingress: _,
            standalone_ineligibility: _,
            construction,
            payloads,
            failed_text_append: _,
            appended_units: _,
            style_order_floor: _,
            token_end: _,
            handler_starts: _,
            main_tree_styleless: _,
            style_subtree: _,
            detached_order: _,
            meta_batches: _,
            #[cfg(test)]
                cancel_after_text_append_chunk: _,
        } = self.storage.take().ok_or(SessionError::Consumed)?;
        let view = match result {
            surgeist_html::construction::HtmlConstructionResult::Document(value) => {
                HtmlConstructionResult::Document(value)
            }
            surgeist_html::construction::HtmlConstructionResult::Fragment(value) => {
                HtmlConstructionResult::Fragment(value)
            }
        };
        Ok(FrozenArena {
            storage: Some(FrozenStorage {
                view,
                control: owner,
                construction,
                lease: Some(lease),
                payloads,
                effects,
                evidence,
                fragment,
            }),
        })
    }
    /// Strict private completed-assembly preflight. Generic native freezing
    /// remains separate; no backing is cloned to test actual ownership.
    pub(crate) fn preflight_completed_freeze(
        &self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if storage
            .evidence
            .as_ref()
            .is_none_or(|evidence| !evidence.retains_diagnostics())
        {
            return Err(SessionError::WrongToken);
        }
        Ok(())
    }
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(control, &self.active()?.control)?;
        self.storage
            .take()
            .ok_or(SessionError::Consumed)?
            .release(control)
    }
}
