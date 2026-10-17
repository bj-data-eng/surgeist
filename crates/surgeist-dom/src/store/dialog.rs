//! HTML Dialog ordinary hooks against the one canonical Draft.
//!
//! Ordinary hooks retain their HTML cursor on the original effect stack. Active
//! setup adds the Dialog to its Document's open list before the watcher wait;
//! inactive setup follows the producer's early return. Source edits publish the
//! reached prefix, while atomic edits refuse external services without publication.
use super::*;
use crate::dialog::{DocumentInteractiveState, DomDialogState};
use surgeist_html::{
    HtmlDialogAction as Action, HtmlDialogCursor, HtmlDialogOperation as Operation,
    HtmlDialogOutcome, HtmlDialogReply as Reply, KnownHtmlAttribute,
};
mod empty_popovers;
mod focusing;
#[cfg(test)]
mod request_close_tests;
#[cfg(test)]
mod show_modal_tests;
mod top_layer;
pub(in crate::store) use focusing::PendingFocusing;

#[derive(Clone, Copy)]
pub(super) enum MethodState {
    Ready,
    AwaitingAttributeHooks,
}
#[derive(Clone, Copy)]
pub(super) struct MethodFrame {
    pub subject: Key,
    pub cursor: HtmlDialogCursor,
    pub state: MethodState,
    pub invocation: source::Invocation,
    pub locals: MethodLocals,
    pub focusing: Option<focusing::FocusingFrame>,
}
#[derive(Clone, Copy)]
pub(super) enum MethodLocals {
    Show {
        captured_document: Option<DocumentId>,
        popover_endpoint: Option<NodeId>,
    },
    Close,
    RequestClose {
        watcher: Option<crate::DomCloseWatcherId>,
    },
}
#[derive(Clone, Copy)]
pub(super) enum MethodInput<'a> {
    Show,
    ShowModal,
    Close(Option<&'a DocumentString>),
    RequestClose(Option<&'a DocumentString>),
}
impl MethodInput<'_> {
    fn operation(self) -> Operation {
        match self {
            Self::Show => Operation::Show,
            Self::ShowModal => Operation::ShowModal,
            Self::RequestClose(_) => Operation::RequestClose,
            Self::Close(result) => Operation::Close {
                result_present: result.is_some(),
            },
        }
    }
    pub(super) fn locals(self) -> MethodLocals {
        match self {
            Self::Show | Self::ShowModal => MethodLocals::Show {
                captured_document: None,
                popover_endpoint: None,
            },
            Self::Close(_) => MethodLocals::Close,
            Self::RequestClose(_) => MethodLocals::RequestClose { watcher: None },
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct HookFrame {
    pub subject: Key,
    pub cursor: HtmlDialogCursor,
    pub invocation: source::Invocation,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct PendingDialog {
    pub subject: Key,
    pub action: Action,
    pub invocation: Option<source::Invocation>,
}

#[derive(Clone, Copy)]
enum DialogList {
    Open,
    Raw,
    Pending,
}

impl Draft {
    pub(super) fn dialog_method_boundary(
        &mut self,
        key: Key,
        input: MethodInput<'_>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.dialog_state(key)?;
        let mut frame = self.dialog_method.unwrap_or(MethodFrame {
            subject: key,
            cursor: HtmlDialogCursor::new(input.operation()),
            state: MethodState::Ready,
            invocation: source::Invocation::Unissued,
            locals: input.locals(),
            focusing: None,
        });
        if frame.cursor.action() == Action::SetModal {
            return self.dialog_modal_phase(frame, cancel);
        }
        if frame.cursor.action() == Action::RunDialogFocusingSteps {
            return self.dialog_focusing_boundary(frame, cancel);
        }
        if matches!(frame.state, MethodState::AwaitingAttributeHooks) {
            self.run_effects(limits, cancel)?;
            frame
                .cursor
                .reply(Reply::Applied)
                .map_err(DomError::DialogProtocol)?;
            frame.state = MethodState::Ready;
        }
        loop {
            self.source_step(cancel)?;
            let action = frame.cursor.action();
            let reply = match action {
                Action::ReadOpen => Reply::Open(
                    self.control_attribute(key, KnownHtmlAttribute::Open, cancel)?
                        .is_some(),
                ),
                Action::ReadModal => Reply::Modal(self.dialog_state(key)?.is_modal()),
                Action::ReadPopoverShowing => {
                    Reply::PopoverShowing(self.dialog_popover_showing(key, cancel)?)
                }
                Action::Complete(HtmlDialogOutcome::Completed) => {
                    self.dialog_method = None;
                    return Ok(());
                }
                Action::Complete(HtmlDialogOutcome::InvalidState) => {
                    return Err(DomError::DialogInvalidState);
                }
                Action::AddOpen => {
                    frame.state = MethodState::AwaitingAttributeHooks;
                    self.dialog_method = Some(frame);
                    let formation = source::formation::Formation::new(self, cancel)?;
                    let name = source::formation::string("open", formation.as_ref(), cancel)?;
                    let value = source::formation::string("", formation.as_ref(), cancel)?;
                    self.set_attribute(key, None, &name, &value, limits, cancel)?;
                    self.run_effects(limits, cancel)?;
                    frame.state = MethodState::Ready;
                    Reply::Applied
                }
                Action::RemoveOpen => {
                    frame.state = MethodState::AwaitingAttributeHooks;
                    self.dialog_method = Some(frame);
                    let formation = source::formation::Formation::new(self, cancel)?;
                    let name = source::formation::string("open", formation.as_ref(), cancel)?;
                    self.remove_attribute(key, None, &name, limits, cancel)?;
                    self.run_effects(limits, cancel)?;
                    frame.state = MethodState::Ready;
                    Reply::Applied
                }
                Action::ClearModal => {
                    self.set_dialog_modal(key, false, cancel)?;
                    Reply::Applied
                }
                Action::RequestTopLayerRemoval => {
                    self.request_top_layer_removal(key, cancel)?;
                    Reply::Applied
                }
                Action::WriteReturnValue => {
                    let MethodInput::Close(Some(result)) = input else {
                        return Err(DomError::InvalidSourceOperation);
                    };
                    self.set_dialog_return_value(key, result, limits, cancel)?;
                    Reply::Applied
                }
                Action::ReadConnected => Reply::Connected(self.shadow_connected(key, cancel)?),
                Action::ReadFullyActive => Reply::FullyActive(
                    self.document_fully_active(self.node(key)?.document, cancel)?,
                ),
                Action::CaptureCloseWatcher => {
                    let watcher = self
                        .dialog_state(key)?
                        .watcher
                        .ok_or(DomError::InvalidSourceOperation)?;
                    frame.locals = MethodLocals::RequestClose {
                        watcher: Some(watcher),
                    };
                    Reply::Applied
                }
                Action::SetForceCloseWatcherEnabled(value) => {
                    self.set_dialog_request_flag(key, value, cancel)?;
                    Reply::Applied
                }
                Action::WriteRequestCloseReturnValue => {
                    let MethodInput::RequestClose(value) = input else {
                        return Err(DomError::InvalidSourceOperation);
                    };
                    self.set_dialog_request_return_value(key, value, limits, cancel)?;
                    Reply::Applied
                }
                Action::WriteNullRequestCloseSource | Action::ClearRequestCloseSource => {
                    self.set_dialog_request_source_null(key, cancel)?;
                    Reply::Applied
                }
                Action::ClearRequestCloseReturnValue => {
                    self.set_dialog_request_return_value(key, None, limits, cancel)?;
                    Reply::Applied
                }
                Action::ReadPreviousFocusPresent => Reply::PreviousFocusPresent(
                    self.element(key)?
                        .common_state()
                        .html()
                        .ok_or(DomError::DialogInapplicable)?
                        .previously_focused_element()
                        .is_some(),
                ),
                Action::CaptureNodeDocument => {
                    frame.locals = MethodLocals::Show {
                        captured_document: Some(self.dialog_method_node_document(key, cancel)?),
                        popover_endpoint: None,
                    };
                    Reply::Applied
                }
                Action::FindTopmostPopoverAncestor => {
                    // This algorithm reads the receiver's then-current Document,
                    // independently of the show-local Document captured above.
                    let document = self.dialog_method_node_document(key, cancel)?;
                    if !self.dialog_method_known_empty_popovers(document, cancel)? {
                        self.dialog_method = Some(frame);
                        return Err(DomError::DialogProcessingUnavailable { action });
                    }
                    let MethodLocals::Show {
                        captured_document, ..
                    } = frame.locals
                    else {
                        return Err(DomError::InvalidSourceOperation);
                    };
                    frame.locals = MethodLocals::Show {
                        captured_document,
                        popover_endpoint: None,
                    };
                    Reply::Applied
                }
                Action::HidePopoversUntil {
                    focus_previous_element: false,
                    fire_events: true,
                } => {
                    let MethodLocals::Show {
                        captured_document: Some(document),
                        popover_endpoint,
                    } = frame.locals
                    else {
                        self.dialog_method = Some(frame);
                        return Err(DomError::DialogProcessingUnavailable { action });
                    };
                    if popover_endpoint.is_some()
                        || !self.dialog_method_known_empty_popovers(document, cancel)?
                    {
                        self.dialog_method = Some(frame);
                        return Err(DomError::DialogProcessingUnavailable { action });
                    }
                    // The captured Document's Hint filter is empty, then its
                    // Auto filter is empty. Neither loop has an item to hide.
                    Reply::Applied
                }
                Action::FireBeforeToggle { .. } => {
                    self.dialog_method = Some(frame);
                    return Err(DomError::NeedsEventfulOperation {
                        subject: self.published_id(key),
                        action,
                    });
                }
                Action::RunDialogFocusingSteps => {
                    return self.dialog_focusing_boundary(frame, cancel);
                }
                _ => {
                    self.dialog_method = Some(frame);
                    return Err(DomError::DialogProcessingUnavailable { action });
                }
            };
            frame
                .cursor
                .reply(reply)
                .map_err(DomError::DialogProtocol)?;
        }
    }

    fn dialog_popover_showing(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        self.dialog_state(key)?;
        Ok(!self
            .element(key)?
            .common_state()
            .html()
            .ok_or(DomError::DialogInapplicable)?
            .popover_is_hidden())
    }
    pub(in crate::store) fn dialog_state(&self, key: Key) -> Result<&DomDialogState, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Err(DomError::DialogInapplicable);
        };
        element.dialog_state().ok_or(DomError::DialogInapplicable)
    }
    fn is_dialog(&self, key: Key) -> Result<bool, DomError> {
        Ok(matches!(&self.node(key)?.kind,
            DomNodeKind::Element(element) if element.dialog_state().is_some()))
    }

    pub(in crate::store) fn admit_dialog_state_copy(
        &self,
        key: Key,
        formation: Option<&source::formation::Formation>,
        keep_return: bool,
        keep_request: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(formation) = formation {
            let state = self.dialog_state(key)?;
            let mut payload = source::storage::Payload::default();
            payload.add_bytes(size_of::<DomDialogState>())?;
            if keep_return {
                payload.string(state.return_value())?;
            }
            if keep_request && let Some(value) = &state.request_return_value {
                payload.string(value)?;
            }
            formation.retain_payload(payload, cancel)?;
        }
        Ok(())
    }
    pub(super) fn set_dialog_return_value(
        &mut self,
        key: Key,
        value: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let old = self.dialog_state(key)?.return_value();
        let mut equal = old.len() == value.len();
        if equal {
            for offset in 0..value.len() {
                self.source_step(cancel)?;
                if old.code_units()[offset] != value.code_units()[offset] {
                    equal = false;
                    break;
                }
            }
        }
        if equal {
            return Ok(());
        }
        let units = self
            .units
            .checked_sub(old.len())
            .and_then(|n| n.checked_add(value.len()))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;

        // DocumentString clones share their immutable actual backing. The new
        // state moves this value; it does not deep-clone the old string or use
        // Arc::make_mut. Preadmit the complete new occurrence while old state and
        // any source operands remain live, before the Arc payload allocation.
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(key, formation.as_ref(), false, true, cancel)?;
        self.source_step(cancel)?;
        let value = source::formation::share(value, formation.as_ref(), cancel)?;
        self.source_step(cancel)?;
        let next = self.dialog_state(key)?.with_return_value(value);
        self.element_mut(key).set_dialog_state(next);
        self.units = units;
        // Canonical Draft storage now owns the admitted occurrence.
        drop(formation);
        self.source_admit(0, cancel)?;
        self.source_admit_units(0, cancel)
    }

    pub(in crate::store) fn set_dialog_request_flag(
        &mut self,
        key: Key,
        value: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self.dialog_state(key)?.force_watcher_enabled == value {
            return Ok(());
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(key, formation.as_ref(), true, true, cancel)?;
        let mut next = self.dialog_state(key)?.clone();
        next.force_watcher_enabled = value;
        self.element_mut(key).set_dialog_state(next);
        drop(formation);
        self.source_admit(0, cancel)
    }
    pub(in crate::store) fn set_dialog_request_source_null(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self.dialog_state(key)?.request_source.is_none() {
            return Ok(());
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(key, formation.as_ref(), true, true, cancel)?;
        let mut next = self.dialog_state(key)?.clone();
        next.request_source = None;
        self.element_mut(key).set_dialog_state(next);
        drop(formation);
        self.source_admit(0, cancel)
    }
    pub(in crate::store) fn set_dialog_request_return_value(
        &mut self,
        key: Key,
        value: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let old = self.dialog_state(key)?.request_return_value.as_ref();
        let equal = match (old, value) {
            (None, None) => true,
            (Some(old), Some(value)) => {
                let mut equal = old.len() == value.len();
                if equal {
                    for (a, b) in old.code_units().iter().zip(value.code_units()) {
                        self.source_step(cancel)?;
                        if a != b {
                            equal = false;
                            break;
                        }
                    }
                }
                equal
            }
            _ => false,
        };
        if equal {
            return Ok(());
        }
        let units = self
            .units
            .checked_sub(old.map_or(0, DocumentString::len))
            .and_then(|n| n.checked_add(value.map_or(0, DocumentString::len)))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(key, formation.as_ref(), true, false, cancel)?;
        let value = value
            .map(|value| source::formation::share(value, formation.as_ref(), cancel))
            .transpose()?;
        let next = self.dialog_state(key)?.with_request_return_value(value);
        self.element_mut(key).set_dialog_state(next);
        self.units = units;
        drop(formation);
        self.source_admit(0, cancel)?;
        self.source_admit_units(0, cancel)
    }

    pub(in crate::store) fn clear_dialog_toggle_tracker(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.dialog_state(key)?;
        // The initial rebase owns the current state occurrences, but this
        // replacement creates another immutable body before the old one drops.
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(key, formation.as_ref(), false, true, cancel)?;
        self.source_step(cancel)?;
        let mut next = self.dialog_state(key)?.clone();
        next.toggle_tracker = None;
        self.element_mut(key).set_dialog_state(next);
        // The Draft now owns the replacement; its normal recount takes over.
        drop(formation);
        self.source_admit(0, cancel)
    }

    pub(in crate::store) fn set_dialog_watcher(
        &mut self,
        key: Key,
        watcher: Option<crate::DomCloseWatcherId>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self.dialog_state(key)?.watcher == watcher {
            return Ok(());
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(key, formation.as_ref(), false, true, cancel)?;
        self.source_step(cancel)?;
        let mut state = self.dialog_state(key)?.clone();
        state.watcher = watcher;
        self.element_mut(key).set_dialog_state(state);
        drop(formation);
        self.source_admit(0, cancel)
    }
    fn dialog_list(&self, document: Key, list: DialogList) -> Result<&[Key], DomError> {
        match self.node(document)?.interactive_document.as_ref() {
            Some(DocumentInteractiveState::Known {
                open_dialogs,
                top_layer,
                pending_top_layer_removals,
            }) => Ok(match list {
                DialogList::Open => open_dialogs,
                DialogList::Raw => top_layer,
                DialogList::Pending => pending_top_layer_removals,
            }),
            Some(DocumentInteractiveState::Unavailable) => {
                Err(DomError::DialogDocumentStateUnavailable)
            }
            None => Err(DomError::WrongNodeKind),
        }
    }
    fn dialog_list_mut(&mut self, document: Key, list: DialogList) -> &mut Vec<Key> {
        let Some(DocumentInteractiveState::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = self.node_mut(document).interactive_document.as_mut()
        else {
            unreachable!("qualified Document list")
        };
        match list {
            DialogList::Open => open_dialogs,
            DialogList::Raw => top_layer,
            DialogList::Pending => pending_top_layer_removals,
        }
    }
    fn dialog_list_contains(
        &self,
        document: Key,
        member: Key,
        list: DialogList,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let members = self.dialog_list(document, list)?;
        for candidate in members {
            self.source_step(cancel)?;
            if *candidate == member {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn dialog_list_remove(
        &mut self,
        document: Key,
        member: Key,
        list: DialogList,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut index = 0;
        loop {
            self.source_step(cancel)?;
            let len = self.dialog_list(document, list)?.len();
            if index >= len {
                break;
            }
            self.source_step(cancel)?;
            let members = self.dialog_list(document, list)?;
            if members[index] != member {
                index += 1;
                continue;
            }
            // Preadmit the bounded structural movement before Vec::remove.
            // No callback or reentry occurs between admission and that effect.
            for _ in index + 1..members.len() {
                self.source_step(cancel)?;
            }
            check_cancel(cancel)?;
            self.dialog_list_mut(document, list).remove(index);
        }
        Ok(())
    }

    pub(super) fn schedule_dialog_hook(
        &mut self,
        key: Key,
        operation: Operation,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !self.is_dialog(key)? {
            return Ok(());
        }
        self.push_effect(
            select::Effect::DialogHook(HookFrame {
                subject: key,
                cursor: HtmlDialogCursor::new(operation),
                invocation: source::Invocation::Unissued,
            }),
            self.effect_scoped,
            cancel,
        )?;
        Ok(())
    }
    pub(super) fn dialog_hook(
        &mut self,
        key: Key,
        operation: Operation,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.schedule_dialog_hook(key, operation, cancel)?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn pending_dialog_hook(&self) -> Option<PendingDialog> {
        match self.effects.last().map(|frame| &frame.effect) {
            Some(select::Effect::DialogHook(frame))
                if matches!(
                    frame.cursor.action(),
                    Action::EstablishCloseWatcher | Action::DestroyCloseWatcher
                ) =>
            {
                Some(PendingDialog {
                    subject: frame.subject,
                    action: frame.cursor.action(),
                    invocation: Some(frame.invocation),
                })
            }
            _ => None,
        }
    }
    pub(super) fn pending_dialog(&self, action: Action) -> Option<PendingDialog> {
        self.pending_dialog_hook()
            .or_else(|| {
                self.dialog_method.map(|frame| PendingDialog {
                    subject: frame.subject,
                    action: frame.cursor.action(),
                    invocation: matches!(frame.cursor.action(), Action::RememberFocusedElement)
                        .then_some(frame.invocation),
                })
            })
            .filter(|pending| pending.action == action)
    }
    fn dialog_list_add(
        &mut self,
        document: Key,
        member: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.dialog_list_contains(document, member, DialogList::Open, cancel)? {
            return Ok(());
        }
        self.dialog_list_append(document, member, DialogList::Open, cancel)
    }
    fn dialog_list_append(
        &mut self,
        document: Key,
        member: Key,
        list: DialogList,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.dialog_list(document, list)?;
        let members = self.dialog_list_mut(document, list);
        let (length, capacity) = (members.len(), members.capacity());
        let admitted = self.source_reserve_amount::<Key>(length, capacity, 1, cancel)?;
        source::reserve_bank(self.dialog_list_mut(document, list), 1, admitted)?;
        self.source_admit(0, cancel)?;
        self.source_step(cancel)?;
        self.dialog_list_mut(document, list).push(member);
        self.source_admit(0, cancel)
    }
    pub(super) fn dialog_hook_effect(
        &mut self,
        mut frame: HookFrame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let key = frame.subject;
        loop {
            self.source_step(cancel)?;
            let action = frame.cursor.action();
            let reply = match action {
                Action::Complete(HtmlDialogOutcome::Completed) => return Ok(()),
                Action::ReadOpen => Reply::Open(
                    self.control_attribute(key, KnownHtmlAttribute::Open, cancel)?
                        .is_some(),
                ),
                Action::ReadModal => Reply::Modal(self.dialog_state(key)?.is_modal()),
                Action::ReadFullyActive => Reply::FullyActive(
                    self.document_fully_active(self.node(key)?.document, cancel)?,
                ),
                Action::ReadConnected => Reply::Connected(self.shadow_connected(key, cancel)?),
                Action::ReadWatcherPresent => {
                    Reply::WatcherPresent(self.dialog_state(key)?.has_close_watcher())
                }
                Action::ReadTopLayerContains => {
                    Reply::TopLayerContains(self.dialog_list_contains(
                        self.node(key)?.document,
                        key,
                        DialogList::Raw,
                        cancel,
                    )?)
                }
                Action::AddOpenDialog => {
                    self.dialog_list_add(self.node(key)?.document, key, cancel)?;
                    Reply::Applied
                }
                Action::RemoveOpenDialog => {
                    self.dialog_list_remove(
                        self.node(key)?.document,
                        key,
                        DialogList::Open,
                        cancel,
                    )?;
                    Reply::Applied
                }
                Action::RemoveTopLayerImmediately => {
                    self.remove_top_layer_immediately(key, cancel)?;
                    Reply::Applied
                }
                Action::ClearModal => {
                    self.set_dialog_modal(key, false, cancel)?;
                    Reply::Applied
                }
                Action::ClearCloseWatcher => {
                    self.set_dialog_watcher(key, None, cancel)?;
                    Reply::Applied
                }
                Action::EstablishCloseWatcher | Action::DestroyCloseWatcher => {
                    if self.headroom_external_hook(cancel)? {
                        Reply::Applied
                    } else {
                        self.push_effect(
                            select::Effect::DialogHook(frame),
                            self.effect_scoped,
                            cancel,
                        )?;
                        return Err(DomError::DialogProcessingUnavailable { action });
                    }
                }
                _ => return Err(DomError::DialogProcessingUnavailable { action }),
            };
            frame
                .cursor
                .reply(reply)
                .map_err(DomError::DialogProtocol)?;
        }
    }
    pub(super) fn dialog_attribute_operation(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        old_present: bool,
        new_present: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Operation>, DomError> {
        self.source_step(cancel)?;
        if !self.is_dialog(key)? {
            return Ok(None);
        }
        // Generic exact-name comparison consumes the HTML vocabulary's spelling;
        // the shared Dialog rule owns the resulting attribute operation.
        self.source_step(cancel)?;
        if name.namespace().uri().is_some()
            || !self.source_ascii(
                name.local_name(),
                KnownHtmlAttribute::Open.local_name(),
                cancel,
            )?
        {
            return Ok(None);
        }
        self.source_step(cancel)?;
        Ok(surgeist_html::dialog_attribute_operation(
            name.namespace(),
            KnownHtmlAttribute::Open,
            old_present,
            new_present,
        ))
    }
    pub(super) fn dialog_insertion(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.dialog_hook(key, Operation::Inserted, limits, cancel)
    }
    pub(super) fn dialog_removal(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.dialog_hook(key, Operation::Removed, limits, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Dom, NodeId) {
        let mut dom =
            Dom::new(DomLimits::new(32, 16384, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let dialog = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        (dom, commit.created(&dialog).unwrap().unwrap())
    }

    #[test]
    fn native_open_list_growth_is_unique_and_one_short_preserves_the_original_bank() {
        let cancel = AtomicBool::new(false);
        for short in [true, false] {
            let mut dom =
                Dom::new(DomLimits::new(32, 16384, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let first = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
            let second = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
            batch.set_attribute(&first, "open", "");
            batch.set_attribute(&second, "open", "");
            let commit = dom.apply(batch, &cancel).unwrap();
            let document = commit.document(&document).unwrap();
            let first = commit.created(&first).unwrap().unwrap();
            let second = commit.created(&second).unwrap().unwrap();
            let before = dom.snapshot().stamp();
            let batch = DomBatch::new();
            let generous =
                crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                generous,
                &cancel,
            )
            .unwrap();
            let doc = draft.resolve(document.node().into()).unwrap();
            let first = draft.resolve(first.into()).unwrap();
            let second = draft.resolve(second.into()).unwrap();
            assert!(!draft.document_fully_active(doc, &cancel).unwrap());
            // This exercises the actual canonical list owner independently of
            // setup eligibility; no fake Global/activity/watcher is supplied.
            draft.dialog_list_add(doc, first, &cancel).unwrap();
            let original = draft.dialog_list(doc, DialogList::Open).unwrap().as_ptr();
            let base = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
            let replacement = 2 * size_of::<Key>();
            let exact = base + replacement;
            draft.source_limits = Some(crate::DomSourceLimits::new(
                exact - usize::from(short),
                100_000,
                10_000_000,
                16,
                4096,
                4096,
                1,
            ));
            let result = draft.dialog_list_add(doc, second, &cancel);
            if short {
                assert_eq!(
                    result.unwrap_err(),
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: exact - 1
                    }
                );
                assert_eq!(draft.dialog_list(doc, DialogList::Open).unwrap(), [first]);
                assert_eq!(
                    draft.dialog_list(doc, DialogList::Open).unwrap().as_ptr(),
                    original
                );
                draft.source_limits = Some(generous);
                draft.dialog_list_add(doc, second, &cancel).unwrap();
            } else {
                result.unwrap();
            }
            let Some(DocumentInteractiveState::Known { open_dialogs, .. }) =
                &draft.node(doc).unwrap().interactive_document
            else {
                panic!("native list");
            };
            assert_eq!(open_dialogs.capacity(), 2);
            assert_eq!(open_dialogs.as_slice(), [first, second]);
            let backing = open_dialogs.as_ptr();
            draft.dialog_list_add(doc, first, &cancel).unwrap();
            assert_eq!(
                draft.dialog_list(doc, DialogList::Open).unwrap(),
                [first, second]
            );
            assert_eq!(
                draft.dialog_list(doc, DialogList::Open).unwrap().as_ptr(),
                backing
            );
            draft.source_limits = Some(generous);
            draft
                .remove_attribute(first, None, &"open".into(), dom.limits, &cancel)
                .unwrap();
            assert_eq!(draft.dialog_list(doc, DialogList::Open).unwrap(), [second]);
            assert_eq!(
                draft.element(first).unwrap().attribute(&"open".into()),
                None
            );
            assert!(draft.effects.is_empty());
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
            assert_eq!(dom.snapshot().stamp(), before);
            // Removal cleaned the real retained member before the native
            // negative fully-active check completed the actual HTML hook.
        }
    }

    #[test]
    fn source_setter_admits_actual_old_and_new_payload_overlap_before_retention() {
        let cancel = AtomicBool::new(false);
        for failure in [
            None,
            Some(DomResource::SourceRetainedBytes),
            Some(DomResource::SourceRetainedUnits),
        ] {
            let (dom, dialog) = fixture();
            let before = dom.snapshot();
            let mut units = Vec::with_capacity(4096);
            units.push(0xd800);
            let capacity = units.capacity();
            let value = DocumentString::from_utf16(units);
            let mut batch = DomBatch::new();
            batch.set_dialog_return_value(dialog, value.clone());
            let generous =
                crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                generous,
                &cancel,
            )
            .unwrap();
            // The original canonical owner, Draft and retained command already
            // have actual receipts. The replacement adds one immutable state
            // payload and one heap UTF16 Vec header/backing occurrence; it
            // shares that backing rather than allocating a physical deep copy.
            let base_bytes =
                draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
            let base_units =
                draft.source_storage_units(&cancel).unwrap() + draft.source_other_units;
            let replacement_bytes =
                size_of::<DomDialogState>() + size_of::<Vec<u16>>() + capacity * size_of::<u16>();
            let byte_limit = base_bytes + replacement_bytes
                - usize::from(failure == Some(DomResource::SourceRetainedBytes));
            let unit_limit = base_units + capacity
                - usize::from(failure == Some(DomResource::SourceRetainedUnits));
            draft.source_limits = Some(crate::DomSourceLimits::new(
                byte_limit, unit_limit, 10_000_000, 16, 4096, 4096, 1,
            ));
            let key = draft.resolve(dialog.into()).unwrap();
            let original_units = draft.units;
            let result = draft.apply(&batch.commands[0], dom.limits, &cancel);
            if let Some(resource) = failure {
                assert_eq!(
                    result.unwrap_err(),
                    DomError::ResourceLimit {
                        resource,
                        limit: if resource == DomResource::SourceRetainedBytes {
                            byte_limit
                        } else {
                            unit_limit
                        },
                    }
                );
                assert_eq!(
                    draft.dialog_state(key).unwrap().return_value().code_units(),
                    []
                );
                assert_eq!(draft.units, original_units);
            } else {
                result.unwrap();
                let actual = draft.dialog_state(key).unwrap().return_value();
                assert_eq!(actual.code_units(), [0xd800]);
                assert_eq!(actual.shared_code_units().capacity(), capacity);
                assert!(Arc::ptr_eq(
                    actual.shared_code_units(),
                    value.shared_code_units()
                ));
                assert_eq!(draft.units, original_units + 1);
            }
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(
                before
                    .node(dialog)
                    .unwrap()
                    .as_dialog()
                    .unwrap()
                    .return_value()
                    .code_units(),
                []
            );
        }
    }

    #[test]
    fn cancelled_source_setter_keeps_state_and_releases_all_local_custody() {
        let (dom, dialog) = fixture();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        batch.set_dialog_return_value(dialog, "unpublished");
        let limits =
            crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
        let (mut draft, _, _) =
            Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel).unwrap();
        let original = draft.units;
        assert_eq!(
            draft
                .apply(&batch.commands[0], dom.limits, &AtomicBool::new(true))
                .unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(draft.units, original);
        let key = draft.resolve(dialog.into()).unwrap();
        assert_eq!(
            draft.dialog_state(key).unwrap().return_value().code_units(),
            []
        );
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    }
    #[test]
    fn source_tracker_clear_admits_candidate_body_before_replacing_state() {
        let cancel = AtomicBool::new(false);
        for short in [true, false] {
            let (mut dom, dialog) = fixture();
            let mut batch = DomBatch::new();
            batch.set_dialog_return_value(dialog, "retained");
            dom.apply(batch, &cancel).unwrap();
            let before = dom.snapshot();
            let batch = DomBatch::new();
            let generous =
                crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                generous,
                &cancel,
            )
            .unwrap();
            let key = draft.resolve(dialog.into()).unwrap();
            let original = draft.dialog_state(key).unwrap() as *const DomDialogState;
            let original_units = draft.units;
            let original_work = draft.source_work.load(Ordering::Relaxed);
            let base = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
            // Canonical and Source Draft state already exist. The helper adds
            // exactly one candidate immutable body; its string clone shares
            // existing backing. No task/tracker or completed Show is invented.
            let exact = base + size_of::<DomDialogState>();
            let tight = exact - usize::from(short);
            draft.source_limits = Some(crate::DomSourceLimits::new(
                tight, 100_000, 10_000_000, 16, 4096, 4096, 1,
            ));
            let result = draft.clear_dialog_toggle_tracker(key, &cancel);
            if short {
                assert_eq!(
                    result.unwrap_err(),
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: tight,
                    }
                );
                assert!(std::ptr::eq(draft.dialog_state(key).unwrap(), original));
                assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
                draft.source_limits = Some(crate::DomSourceLimits::new(
                    exact, 100_000, 10_000_000, 16, 4096, 4096, 1,
                ));
                draft.clear_dialog_toggle_tracker(key, &cancel).unwrap();
            } else {
                result.unwrap();
            }
            assert!(!std::ptr::eq(draft.dialog_state(key).unwrap(), original));
            assert_eq!(draft.dialog_state(key).unwrap().toggle_tracker, None);
            assert_eq!(
                draft.dialog_state(key).unwrap().return_value().code_units(),
                [114, 101, 116, 97, 105, 110, 101, 100]
            );
            assert_eq!(draft.units, original_units);
            assert!(draft.source_work.load(Ordering::Relaxed) > original_work);
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
            assert_eq!(
                draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes,
                base,
                "the transferred replacement has the original retained shape"
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(
                before
                    .node(dialog)
                    .unwrap()
                    .as_dialog()
                    .unwrap()
                    .return_value()
                    .code_units(),
                [114, 101, 116, 97, 105, 110, 101, 100]
            );
        }
    }

    #[test]
    fn cancelled_source_tracker_clear_preserves_original_state_for_same_draft_retry() {
        let cancel = AtomicBool::new(false);
        let (dom, dialog) = fixture();
        let before = dom.snapshot();
        let batch = DomBatch::new();
        let limits =
            crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
        let (mut draft, _, _) =
            Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel).unwrap();
        let key = draft.resolve(dialog.into()).unwrap();
        let original = draft.dialog_state(key).unwrap() as *const DomDialogState;
        let original_units = draft.units;
        assert_eq!(
            draft.clear_dialog_toggle_tracker(key, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        );
        assert!(std::ptr::eq(draft.dialog_state(key).unwrap(), original));
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        draft.clear_dialog_toggle_tracker(key, &cancel).unwrap();
        assert!(!std::ptr::eq(draft.dialog_state(key).unwrap(), original));
        assert_eq!(draft.dialog_state(key).unwrap().toggle_tracker, None);
        assert!(draft.dialog_state(key).unwrap().return_value().is_empty());
        assert_eq!(draft.units, original_units);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }

    #[test]
    fn equal_source_value_admits_each_comparison_without_mutation_or_custody() {
        let cancel = AtomicBool::new(false);
        let value = DocumentString::from_utf16(vec![0xd800, 0, 13, 10, 0xdc00]);
        let compared = value.len() as u64;
        for work in [compared, compared + 1] {
            let (mut dom, dialog) = fixture();
            let mut batch = DomBatch::new();
            batch.set_dialog_return_value(dialog, value.clone());
            dom.apply(batch, &cancel).unwrap();
            let before = dom.snapshot();
            let batch = DomBatch::new();
            let generous =
                crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                generous,
                &cancel,
            )
            .unwrap();
            let key = draft.resolve(dialog.into()).unwrap();
            let original_units = draft.units;
            let original = draft.dialog_state(key).unwrap().return_value().clone();
            // One actual entry action plus one admitted read for each of the
            // five literal UTF16 pairs. Begin a fresh measured setter interval;
            // initial Source formation was separately admitted above.
            draft.source_limits = Some(crate::DomSourceLimits::new(
                16_000_000, 100_000, work, 16, 4096, 4096, 1,
            ));
            draft.source_work.store(0, Ordering::Relaxed);
            let result = draft.set_dialog_return_value(key, &value, dom.limits, &cancel);
            if work == compared {
                assert_eq!(
                    result.unwrap_err(),
                    DomError::SourceWorkLimit { limit: work }
                );
            } else {
                result.unwrap();
            }
            assert_eq!(draft.source_work.load(Ordering::Relaxed), work);
            let retained = draft.dialog_state(key).unwrap().return_value();
            assert_eq!(retained.code_units(), [0xd800, 0, 13, 10, 0xdc00]);
            assert!(Arc::ptr_eq(
                retained.shared_code_units(),
                original.shared_code_units()
            ));
            assert_eq!(draft.units, original_units);
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
        }
    }
}
