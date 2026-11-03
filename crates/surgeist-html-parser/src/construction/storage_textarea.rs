//! Textarea raw value state and its reset from child text.

use super::*;
use surgeist_html::{construction::HtmlNodeKind, elements::form::HtmlTextareaRawValueAction};

impl SessionStorage {
    pub(super) fn textarea_slot(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<Option<(usize, TextareaState)>, SessionError> {
        find_textarea_state(&self.payloads, control, node)
    }

    /// Child text is direct Text children in the actual source view. A moved
    /// root is excluded for removal, even if the fixed final links reinsert it.
    pub(super) fn refresh_textarea(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        reset: bool,
    ) -> Result<(), SessionError> {
        let Some((slot, mut state)) = self.textarea_slot(control, node)? else {
            return Ok(());
        };
        let old = textarea_raw(&self.payloads, control, state.raw)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if Arc::strong_count(old.shared_code_units()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        if reset {
            payload_write::<TextareaState>(control, 1)?;
            state.flags.reset();
            let Some(RetainedPayload::Textarea { state: stored, .. }) =
                self.payloads.values.get_mut(slot)
            else {
                return Err(latch_registry_row(control));
            };
            *stored = state;
        } else if state.flags.children_changed() == HtmlTextareaRawValueAction::Keep {
            return Ok(());
        }
        let dispose_previous = self.releasable_string(control, state.raw, false)?;
        let mut raw = Utf16Buffer::new(control)?;
        let mut group = None;
        let mut units = 0usize;
        let copied = (|| {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            for position in 0..self
                .construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(control))?
                .child_count()
            {
                control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                let child = self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_at(position)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                if Some(child) != detached
                    && let HtmlNodeKind::Text(value) = &self
                        .construction
                        .node_at(child)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                {
                    raw.extend_units(value.code_units(), control)?;
                    let added = value.len();
                    let end = units.checked_add(added).ok_or_else(|| {
                        control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                    })?;
                    let source = self.field_slot(control, StringLocation::Text(child))?;
                    let mut incoming = self.copied_slot_group(control, source)?;
                    if incoming.is_none() && self.evidence.is_some() {
                        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
                        let fresh = evidence.source_independent_group(control, added)?;
                        evidence.bind_leaf_value(control, &fresh, source)?;
                        incoming = Some(fresh);
                    }
                    if let Some(incoming) = incoming {
                        group = Some(if let Some(previous) = group.take() {
                            self.evidence
                                .as_mut()
                                .ok_or(SessionError::WrongToken)?
                                .merge_groups(control, &previous, &incoming, units, end)?
                        } else {
                            incoming
                        });
                    }
                    units = end;
                }
            }
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = copied {
            raw.release(control)?;
            return Err(error);
        }
        let group = match self.reset_group(control, state.raw, group, units) {
            Ok(group) => group,
            Err(error) => {
                raw.release(control)?;
                return Err(error);
            }
        };
        let raw = raw.freeze(control)?;
        let (native, next) = self.register_group_string_owned(control, raw, group)?;
        drop(native);
        // All useful work and both actual record moves precede a fixed commit.
        // The old raw has no native/metadata Arc alias: metadata holds an index.
        payload_write::<TextareaState>(control, 1)?;
        payload_write::<RetainedPayload>(control, 2)?;
        self.retire_field_group(control, state.raw)?;
        let Some(RetainedPayload::Textarea { state: stored, .. }) =
            self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        let previous = state.raw;
        state.raw = next;
        *stored = state;
        if dispose_previous {
            self.dispose_aggregate_string(control, previous)?;
        }
        Ok(())
    }
}
