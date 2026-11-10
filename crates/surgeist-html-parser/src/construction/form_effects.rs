//! Form association effects of tree and attribute changes.

use super::*;
use surgeist_html::{
    bounded::form::{
        HtmlFormOwnerResetDecision, HtmlFormOwnerResetTrigger, form_owner_reset_trigger,
    },
    construction::ConstructionNodeKind,
    elements::form::HtmlNativeFormAssociationKind,
    values::DocumentString,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FormEffectStage {
    Removal,
    RemovalChildren,
    Adoption,
    Insertion,
    InsertionChildren,
    /// The children changed because text was appended to the end of the last
    /// child of the parent, and by nothing else.
    AppendChildren,
    IdChange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FormEffectPass {
    Local,
    Intrinsic,
    Global {
        /// Position in the registry's list of rows of listed controls with a
        /// form attribute.
        next_form: usize,
        tree: Option<usize>,
    },
}

/// Fixed, retained progress over the same actual tree and payload registry.
/// A stopped prefix is for cleanup only and cannot be resumed or frozen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FormEffects {
    pub(super) root: usize,
    pub(super) old_parent: Option<usize>,
    pub(super) new_parent: Option<usize>,
    pub(super) stage: FormEffectStage,
    pub(super) insert: bool,
    pub(super) cause: InsertionCause,
    pub(super) next: Option<usize>,
    pub(super) pass: FormEffectPass,
    /// Whether a base element with an `href` in the subtree has yet to have its hook run.
    pub(super) href_base: base::HrefBaseAhead,
}

impl SessionStorage {
    fn nearest_form(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let mut next = self.ordinary_parent(control, node, detached)?;
        for _ in 0..self.construction.storage(control)?.len() {
            let Some(node) = next else { return Ok(None) };
            if self.html_local(control, node, "form")? {
                return Ok(Some(node));
            }
            next = self.ordinary_parent(control, node, detached)?;
        }
        Err(SessionError::Cycle)
    }

    pub(super) fn prepare_form_effects(
        &self,
        control: &mut ParseControl,
        root: usize,
        remove: bool,
        insert: bool,
        new_parent: Option<usize>,
    ) -> Result<Option<FormEffects>, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let old_parent = self
            .construction
            .node_at(root)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .map(|node| node.position());
        let mut children_changed = false;
        for parent in [old_parent.filter(|_| remove), new_parent]
            .into_iter()
            .flatten()
        {
            if self.textarea_slot(control, parent)?.is_some()
                || self.html_local(control, parent, "script")?
                || self.html_local(control, parent, "style")?
            {
                children_changed = true;
            }
        }
        let adoption = if let Some(parent) = new_parent {
            match (
                self.node_document_ref(control, root),
                self.node_document_ref(control, parent),
            ) {
                (Ok(old), Ok(new)) => old != new,
                (Err(SessionError::NodeDocumentUnavailable), _)
                | (_, Err(SessionError::NodeDocumentUnavailable)) => false,
                (Err(error), _) | (_, Err(error)) => return Err(error),
            }
        } else {
            false
        };
        let mut local = adoption;
        // The subtree is local when it holds a Base or form association row's
        // node, or an id while some listed control has a form attribute (an id
        // can then resolve that attribute). One walk of the subtree finds the
        // rows through their joins and whether it holds an id, so only a
        // subtree with an id scans the registry's form associations.
        let mut joined = false;
        let mut identified = false;
        let mut next = Some(root);
        while let Some(node) = next {
            for name in [
                "option",
                "optgroup",
                "selectedcontent",
                "script",
                "details",
                "dialog",
                "img",
                "source",
                "meta",
                "style",
                "link",
            ] {
                if self.html_local(control, node, name)? {
                    local = true;
                    if name == "option" || name == "selectedcontent" {
                        self.select_metadata(control, node)?;
                    }
                    if name == "script" {
                        script_metadata(&self.payloads, control, node)?;
                    }
                }
            }
            if self.svg_script_local(control, node)? {
                self.svg_state_at(control, node)?;
                local = true;
            }
            for join in [Join::Base, Join::FormAssociation] {
                joined |= self.payloads.joined(control, node, join)?.is_some();
            }
            identified |= self.has_id(control, node)?;
            next = self.next_effect_node(control, root, node)?;
        }
        let mut listed_explicit = false;
        if !joined && identified {
            #[cfg(test)]
            self.assert_explicit_forms_exact();
            for &slot in self.payloads.explicit_form_list() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if let Some(RetainedPayload::FormAssociation {
                    node: Some(node),
                    state,
                }) = self.payloads.values.get(slot)
                    && state.kind.listed()
                    && self.form_attribute(control, *node, "form")?.is_some()
                {
                    listed_explicit = true;
                    break;
                }
            }
        }
        #[cfg(test)]
        assert_eq!(
            joined || listed_explicit,
            self.uncharged_form_subtree_is_local(root),
            "form locality of the subtree at {root}"
        );
        local |= joined || listed_explicit;
        if !local && !children_changed {
            return Ok(None);
        }
        Ok(Some(FormEffects {
            root,
            old_parent,
            new_parent,
            stage: if remove {
                FormEffectStage::Removal
            } else if adoption {
                FormEffectStage::Adoption
            } else {
                FormEffectStage::Insertion
            },
            insert,
            cause: InsertionCause::NonParser,
            next: Some(root),
            pass: FormEffectPass::Local,
            href_base: base::HrefBaseAhead::Unscanned,
        }))
    }

    /// Borrowed exact attributes over the actual managed element. Neither the
    /// value nor the installed attribute container is cloned for resolution.
    pub(super) fn form_attribute<'a>(
        &'a self,
        control: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<Option<&'a DocumentString>, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let record = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        let ConstructionNodeKind::Element(element) = record.kind() else {
            return Ok(None);
        };
        for attribute in element.attributes().iter() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            if attribute.name().namespace().uri().is_none()
                && exact_ascii(control, Some(attribute.name().local_name()), local)?
            {
                return Ok(Some(attribute.value()));
            }
        }
        Ok(None)
    }

    fn has_id(&self, control: &mut ParseControl, node: usize) -> Result<bool, SessionError> {
        let id = self.form_attribute(control, node, "id")?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(id.is_some_and(|id| !id.is_empty()))
    }

    fn first_id_form(
        &self,
        control: &mut ParseControl,
        node: usize,
        value: &DocumentString,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let root = self.ordinary_root(control, node, detached)?;
        let mut next = Some(root);
        while let Some(candidate) = next {
            if let Some(id) = self.form_attribute(control, candidate, "id")? {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if !id.is_empty() && exact_string_equal(control, id, value)? {
                    return Ok(self
                        .html_local(control, candidate, "form")?
                        .then_some(candidate));
                }
            }
            next = self.next_view_node(control, root, candidate, detached)?;
        }
        Ok(None)
    }

    pub(super) fn reset_form_association(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        node: usize,
        mut state: FormAssociationState,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        use surgeist_html::bounded::form::{
            HtmlFormOwnerResetAction as Action, HtmlFormOwnerResetCursor,
            HtmlFormOwnerResetReply as Reply,
        };
        // Admit the actual constant-state rule before retaining it. This cursor
        // has no identity or tree: every effect below mutates the same owner.
        payload_write::<HtmlFormOwnerResetCursor>(control, 1)?;
        let mut cursor = HtmlFormOwnerResetCursor::new(state.kind.listed(), state.owner.is_some());
        payload_write::<Option<usize>>(control, 1)?;
        let mut found = None;
        loop {
            let reply = match cursor.action() {
                Action::ClearParserInserted => {
                    // Source step 1 precedes every attribute/ancestor lookup.
                    state.parser_inserted = false;
                    state.evidence = HtmlParseFormEvidence::Reset;
                    set_form_association_state(self, control, slot, state)?;
                    Reply::Applied
                }
                Action::ReadFormAttributePresence => Reply::FormAttributePresent(
                    self.form_attribute(control, node, "form")?.is_some(),
                ),
                Action::ReadCurrentOwnerIsNearestAncestor => {
                    let Some(owner) = state.owner else {
                        return Err(latch_registry_row(control));
                    };
                    Reply::CurrentOwnerIsNearestAncestor(
                        self.nearest_form(control, node, detached)? == Some(owner),
                    )
                }
                Action::ClearOwner => {
                    state.owner = None;
                    set_form_association_state(self, control, slot, state)?;
                    Reply::Applied
                }
                Action::ApplyOwnerChangeEffects => {
                    if state.kind == HtmlNativeFormAssociationKind::Input {
                        self.normalize_radio(control, node, detached)?;
                    }
                    Reply::Applied
                }
                Action::ReadConnectedness => {
                    Reply::Connected(self.shadow_including_connected(control, node, detached)?)
                }
                Action::FindFirstIdMatch => {
                    let value = self
                        .form_attribute(control, node, "form")?
                        .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                    let candidate = self.first_id_form(control, node, value, detached)?;
                    payload_write::<Option<usize>>(control, 1)?;
                    found = candidate;
                    Reply::FormFound(found.is_some())
                }
                Action::FindNearestAncestor => {
                    // The earlier retention probe cannot supply this candidate:
                    // lookup happens afresh after the owner-clear effects.
                    let candidate = self.nearest_form(control, node, detached)?;
                    payload_write::<Option<usize>>(control, 1)?;
                    found = candidate;
                    Reply::FormFound(found.is_some())
                }
                Action::AssociateFoundOwner => {
                    let Some(owner) = found else {
                        return Err(latch_registry_row(control));
                    };
                    state.owner = Some(owner);
                    set_form_association_state(self, control, slot, state)?;
                    Reply::Applied
                }
                Action::Complete => return Ok(()),
            };
            payload_write::<HtmlFormOwnerResetCursor>(control, 1)?;
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        }
    }

    /// Called synchronously only by the mutation that installed this phase.
    /// No consumer resume method exists. After a stop only disposal is allowed.
    pub(super) fn step_form_effects(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.step_form_effects_with_meta_csp(control, &mut None)
    }
    pub(super) fn step_form_effects_with_meta_csp(
        &mut self,
        control: &mut ParseControl,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let result = (|| {
            control.require_identity(&self.control)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<Option<FormEffects>>() as u64,
            )?;
            let Some(mut effects) = self.effects.form else {
                return Ok(());
            };
            if effects.stage == FormEffectStage::Adoption {
                let parent = effects.new_parent.ok_or(SessionError::NotContainer)?;
                let document = match self.node_document_ref(control, parent) {
                    Ok(document) => Some(document),
                    Err(SessionError::NodeDocumentUnavailable) => None,
                    Err(error) => return Err(error),
                };
                let old = match self.node_document_ref(control, effects.root) {
                    Ok(document) => Some(document),
                    Err(SessionError::NodeDocumentUnavailable) => None,
                    Err(error) => return Err(error),
                };
                effects.stage = FormEffectStage::Insertion;
                effects.next = Some(effects.root);
                effects.pass = FormEffectPass::Local;
                payload_write::<Option<FormEffects>>(control, 1)?;
                self.effects.form = Some(effects);
                if let (Some(old), Some(document)) = (old, document)
                    && old != document
                {
                    self.start_effect(
                        control,
                        effects::EffectFrame::AdoptSubtree {
                            root: effects.root,
                            next: Some(effects.root),
                            document,
                        },
                    )?;
                }
                return Ok(());
            }
            if matches!(
                effects.stage,
                FormEffectStage::RemovalChildren
                    | FormEffectStage::InsertionChildren
                    | FormEffectStage::AppendChildren
            ) {
                let removal = effects.stage == FormEffectStage::RemovalChildren;
                // A fragment's retained insertion owner completes every sibling
                // hook before its single parent hook/static-list continuation.
                if !removal && self.fragment_insertion_parent(control, effects.new_parent)? {
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = None;
                    return Ok(());
                }
                let parent = if removal {
                    effects.old_parent
                } else {
                    effects.new_parent
                };
                if let Some(parent) = parent {
                    let cause = if effects.stage == FormEffectStage::AppendChildren {
                        // The stage and the units are set together.
                        let Some(added) = self.appended_units.take() else {
                            return Err(latch_registry_row(control));
                        };
                        TextareaRefresh::Append { added }
                    } else {
                        TextareaRefresh::Rebuild
                    };
                    self.refresh_textarea(control, parent, removal.then_some(effects.root), cause)?;
                    self.association_children(control, parent, removal.then_some(effects.root))?;
                }
                if removal && effects.insert {
                    effects.stage = FormEffectStage::Adoption;
                    effects.next = Some(effects.root);
                    effects.pass = FormEffectPass::Local;
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    #[cfg(test)]
                    if let Some(cancellation) = self.cancel_after_removal.take() {
                        cancellation.cancel();
                    }
                    if let Some(parent) = parent
                        && self.html_local(control, parent, "script")?
                        && self.connected(control, parent, Some(effects.root))?
                    {
                        self.start_effect(
                            control,
                            effects::EffectFrame::ScriptPreparation {
                                node: parent,
                                detached: Some(effects.root),
                            },
                        )?;
                    }
                    return Ok(());
                }
                payload_write::<Option<FormEffects>>(control, 1)?;
                self.effects.form = None;
                if !removal && effects.insert {
                    // Retain only the finite continuation now. Actual static
                    // collection runs after the source-prior parent script hook.
                    self.start_effect(
                        control,
                        effects::EffectFrame::InsertionPostConnection {
                            roots: effects::PostConnectionRoots::Single(effects.root),
                            cause: effects.cause,
                        },
                    )?;
                }
                if let Some(parent) = parent
                    && self.html_local(control, parent, "script")?
                    && self.connected(control, parent, removal.then_some(effects.root))?
                {
                    self.start_effect(
                        control,
                        effects::EffectFrame::ScriptPreparation {
                            node: parent,
                            detached: removal.then_some(effects.root),
                        },
                    )?;
                }
                return Ok(());
            }
            let Some(node) = effects.next else {
                if effects.stage == FormEffectStage::Removal
                    || effects.stage == FormEffectStage::Insertion
                {
                    effects.stage = if effects.stage == FormEffectStage::Removal {
                        FormEffectStage::RemovalChildren
                    } else {
                        FormEffectStage::InsertionChildren
                    };
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    return Ok(());
                }
                payload_write::<Option<FormEffects>>(control, 1)?;
                self.effects.form = None;
                return Ok(());
            };
            let detached = (effects.stage == FormEffectStage::Removal).then_some(effects.root);
            match effects.pass {
                FormEffectPass::Local => {
                    if let Some((slot, state)) =
                        find_form_association_state(&self.payloads, control, node)?
                    {
                        match effects.stage {
                            FormEffectStage::Removal => {
                                if form_owner_reset_trigger(
                                    HtmlFormOwnerResetTrigger::RemovingOrMoving {
                                        has_owner: state.owner.is_some(),
                                    },
                                ) == HtmlFormOwnerResetDecision::ResetIfOwnerInDifferentTree
                                {
                                    let node_root = self.ordinary_root(control, node, detached)?;
                                    let Some(owner) = state.owner else {
                                        return Err(latch_registry_row(control));
                                    };
                                    if node_root != self.ordinary_root(control, owner, detached)? {
                                        self.reset_form_association(
                                            control, slot, node, state, detached,
                                        )?;
                                    }
                                }
                            }
                            FormEffectStage::Insertion
                                if form_owner_reset_trigger(
                                    HtmlFormOwnerResetTrigger::Insertion {
                                        parser_inserted: state.parser_inserted,
                                    },
                                ) == HtmlFormOwnerResetDecision::Reset =>
                            {
                                self.reset_form_association(control, slot, node, state, None)?;
                            }
                            FormEffectStage::Insertion | FormEffectStage::IdChange => {}
                            // The children stages returned above.
                            FormEffectStage::Adoption
                            | FormEffectStage::RemovalChildren
                            | FormEffectStage::InsertionChildren
                            | FormEffectStage::AppendChildren => {
                                return Err(latch_registry_row(control));
                            }
                        }
                    }
                    effects.pass = FormEffectPass::Intrinsic;
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    return Ok(());
                }
                FormEffectPass::Intrinsic => {
                    let id_event = self.has_id(control, node)?
                        && match effects.stage {
                            // The selected source predicate requires an actual
                            // removing hook and post-removal non-Document tree;
                            // there is no old-connected prerequisite.
                            FormEffectStage::Removal => !self.connected(control, node, detached)?,
                            FormEffectStage::Insertion => self.connected(control, node, None)?,
                            FormEffectStage::IdChange => true,
                            // The children stages returned above.
                            FormEffectStage::Adoption
                            | FormEffectStage::RemovalChildren
                            | FormEffectStage::InsertionChildren
                            | FormEffectStage::AppendChildren => {
                                return Err(latch_registry_row(control));
                            }
                        };
                    if id_event {
                        effects.pass = FormEffectPass::Global {
                            next_form: 0,
                            tree: if effects.stage == FormEffectStage::IdChange {
                                Some(self.ordinary_root(control, node, None)?)
                            } else {
                                None
                            },
                        };
                    } else {
                        effects.pass = FormEffectPass::Local;
                        effects.next = if effects.stage == FormEffectStage::IdChange {
                            None
                        } else {
                            self.next_effect_node(control, effects.root, node)?
                        };
                    }
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    if effects.stage != FormEffectStage::IdChange {
                        self.image_tree_changed(
                            control,
                            node,
                            if effects.stage == FormEffectStage::Removal {
                                surgeist_html::elements::image::HtmlImageTreeChange::Removed
                            } else {
                                surgeist_html::elements::image::HtmlImageTreeChange::Inserted
                            },
                            effects.root,
                            effects.old_parent,
                        )?;
                        self.meta_tree_hook(control, node, detached, &effects, service)?;
                        self.association_tree_hook(
                            control,
                            node,
                            detached,
                            effects.stage == FormEffectStage::Removal,
                            effects.old_parent,
                            effects.root,
                        )?;
                        self.intrinsic_tree_hook(
                            control,
                            node,
                            detached,
                            effects.stage == FormEffectStage::Removal,
                            effects.root,
                        )?;
                    }
                    return Ok(());
                }
                FormEffectPass::Global { next_form, tree } => {
                    #[cfg(test)]
                    self.assert_explicit_forms_exact();
                    let next_slot = self.payloads.explicit_form_list().get(next_form).copied();
                    if let Some(next_slot) = next_slot {
                        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        if let Some(RetainedPayload::FormAssociation {
                            node: Some(candidate),
                            state,
                        }) = self.payloads.values.get(next_slot)
                        {
                            control.charge(
                                HtmlParseWorkCategory::CopyByte,
                                (const {
                                    mem::size_of::<usize>() + mem::size_of::<FormAssociationState>()
                                }) as u64,
                            )?;
                            let candidate = *candidate;
                            let state = *state;
                            // These records can only be issued by the same
                            // fresh ordinary Document creation owner. A
                            // Document ID event also affects its detached
                            // controls; connectivity is tested by reset.
                            if self.node_document_ref(control, candidate)?
                                == self.node_document_ref(control, node)?
                                && form_owner_reset_trigger(HtmlFormOwnerResetTrigger::IdEvent {
                                    listed: state.kind.listed(),
                                }) == HtmlFormOwnerResetDecision::ResetIfFormAttributePresent
                                && self.form_attribute(control, candidate, "form")?.is_some()
                                && match tree {
                                    Some(tree) => {
                                        self.ordinary_root(control, candidate, detached)? == tree
                                    }
                                    None => true,
                                }
                            {
                                self.reset_form_association(
                                    control, next_slot, candidate, state, detached,
                                )?;
                            }
                        }
                        effects.pass = FormEffectPass::Global {
                            next_form: next_form.saturating_add(1),
                            tree,
                        };
                    } else {
                        effects.pass = FormEffectPass::Local;
                        effects.next = if effects.stage == FormEffectStage::IdChange {
                            None
                        } else {
                            self.next_effect_node(control, effects.root, node)?
                        };
                    }
                }
            }
            payload_write::<Option<FormEffects>>(control, 1)?;
            self.effects.form = Some(effects);
            Ok(())
        })();
        match result {
            Err(
                error @ (SessionError::Stop(_)
                | SessionError::CspService
                | SessionError::CspProtocol(_)),
            ) => Err(error),
            Err(error) => Err(latch_effect_fault(control, error)),
            Ok(()) => Ok(()),
        }
    }
}
