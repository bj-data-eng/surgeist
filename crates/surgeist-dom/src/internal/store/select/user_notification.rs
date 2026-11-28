use super::*;
use crate::internal::store::source::operation::element_queue::DomSelectNotificationTask;
use surgeist_html::bounded::select::{
    HtmlSelectNotificationAction as Action, HtmlSelectNotificationReply as Reply,
};

impl Dom {
    /// Apply this notification's current local DOM effect.
    ///
    /// The returned commit establishes the effect before the cursor advances.
    /// A retired receiver returns `None`. Refusal retains the current action.
    pub fn apply_select_notification_local_action(
        &mut self,
        task: &mut DomSelectNotificationTask,
        mut limits: crate::internal::source::DomSourceLimits,
        bank_work: &mut u64,
        cancel: &AtomicBool,
    ) -> Result<Option<DomCommit>, DomError> {
        let id = task.info().id().target();
        let cursor = task.cursor.as_mut().ok_or(DomError::InvalidNativeTask)?;
        let action = cursor.action().ok_or(DomError::InvalidNativeTask)?;
        let reply = match action {
            Action::MarkUserValidity => Reply::UserValiditySet,
            Action::UpdateFallbackLabel => Reply::FallbackLabelUpdated,
            _ => return Err(DomError::InvalidNativeTask),
        };
        limits.work = limits.work.min(*bank_work);
        let initial = source::initial::Initial::new(limits);
        let formed = (|| {
            initial.step(cancel)?;
            self.require_sync_source_idle()?;
            self.qualify_element_task(&task.binding)?;
            initial.retain(
                source::storage::Payload {
                    bytes: size_of::<BatchIdentity>(),
                    units: 0,
                },
                cancel,
            )?;
            Draft::new_formed(
                self,
                source::formation::allocated_arc(BatchIdentity),
                Some(&initial),
                cancel,
            )
        })();
        let mut draft = match formed {
            Ok(draft) => draft,
            Err(error) => {
                *bank_work -= initial.spent_work();
                return Err(error);
            }
        };
        let result = (|| {
            let key = match draft.resolve(id.into()) {
                Ok(key) => key,
                Err(DomError::StaleNode) => return Ok(None),
                Err(error) => return Err(error),
            };
            match action {
                Action::MarkUserValidity => {
                    let mut state = draft.select_state_mut(key, cancel)?;
                    state.flags.mark_user_validity();
                    state.commit(cancel)?;
                }
                Action::UpdateFallbackLabel => {
                    draft.replace_select_fallback_label(
                        &self.snapshot(),
                        id,
                        self.limits,
                        cancel,
                    )?;
                }
                _ => return Err(DomError::InvalidNativeTask),
            }
            let prepared = draft.finish(self, cancel)?;
            if limits.phases == 0 && prepared.changed_from(&self.state) {
                return Err(DomError::SourcePublicationLimit);
            }
            if limits.outputs == 0 {
                return Err(DomError::SourceOutputLimit);
            }
            self.publish(prepared, cancel).map(Some)
        })();
        *bank_work -= draft.source_work.load(Ordering::Relaxed);
        let committed = result?;
        if committed.is_some() {
            cursor.accept(reply).map_err(|_| DomError::SelectProtocol)?;
        }
        Ok(committed)
    }
}

impl Draft {
    fn replace_select_fallback_label(
        &mut self,
        snapshot: &DomSnapshot,
        select: NodeId,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let root = self
            .select_shadow_root(Key::Existing(select))?
            .ok_or(DomError::SelectProtocol)?;
        let slots = crate::internal::slot::SelectSlots::read(self, root, cancel)?;
        let fallback = self
            .node(slots.button)?
            .children()
            .first()
            .ok_or(DomError::SelectProtocol)?;
        let button = self.node(fallback)?;
        let previous = button.children().first();
        let document = button.document();
        let global = button.relevant_global();
        if !button.common().ua_owned || button.children().len() > 1 {
            return Err(DomError::SelectProtocol);
        }

        let formation = source::formation::Formation::new(self, cancel)?;
        let mut label = Vec::new();
        if let Some((_, selected)) = snapshot
            .state
            .first_selected_option_with(select, &mut || self.source_step(cancel))?
        {
            snapshot
                .node_with(selected, || self.source_step(cancel))?
                .as_option()
                .ok_or(DomError::SelectProtocol)?
                .visit_label_with(
                    || self.source_step(cancel),
                    |bytes| {
                        let scratch = formation.as_ref().map(source::formation::Formation::fork);
                        if let Some(funding) = &scratch {
                            funding.retain_bytes(bytes, cancel)?;
                        }
                        Ok(scratch)
                    },
                    |units| {
                        source::formation::reserve_utf16(
                            &mut label,
                            units.len(),
                            formation.as_ref(),
                            cancel,
                        )?;
                        for unit in units {
                            self.source_step(cancel)?;
                            label.push(*unit);
                        }
                        Ok(())
                    },
                )?;
        }
        let after = if label.is_empty() {
            tree::AfterRemovingChildren::Complete
        } else {
            if let Some(funding) = &formation {
                let mut header = source::storage::Payload::default();
                header.empty_string()?;
                funding.retain_payload(header, cancel)?;
            }
            let text = self.allocate_formed(
                NewNode::Text(DocumentString::from_utf16(label)),
                document,
                limits,
                cancel,
                formation,
            )?;
            let mut common = self.common_fields_mut(text, cancel)?;
            common.ua_owned = true;
            common.relevant_global = global;
            common.commit(cancel)?;
            tree::AfterRemovingChildren::Insert(text)
        };
        self.replace_all_children(fallback, after, limits, cancel)?;
        if let Some(previous) = previous {
            self.destroy_detached_tree(previous, limits, cancel)?;
        }
        Ok(())
    }
}
