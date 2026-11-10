//! Textarea raw value state and its reset from child text.

use super::*;
use surgeist_html::{
    construction::ConstructionNodeKind, elements::form::HtmlTextareaRawValueAction,
};

/// Why a textarea's raw value is refreshed from its children.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TextareaRefresh {
    /// The element left the stack of open elements: the dirty flag resets and
    /// the value is rebuilt.
    Reset,
    /// The children changed in a way only a rebuild from every child answers.
    Rebuild,
    /// Text was appended to the end of the last child and nothing else
    /// changed: the new value is the old one with those units at its end.
    Append { added: NonZeroUsize },
}

/// How a refresh produces the new raw value.
enum RawValue {
    /// Copies the text of every child into a new backing.
    Copy(Utf16Buffer),
    /// Appends the units that the last child gained to the end of the old backing.
    Extend(NonZeroUsize),
}
impl RawValue {
    /// Returns what a refused refresh holds to the accounting owner.
    fn release(self, control: &mut ParseControl) -> Result<(), HtmlParseStop> {
        match self {
            Self::Copy(raw) => raw.release(control),
            Self::Extend(_) => Ok(()),
        }
    }
}

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
    ///
    /// An `Append` refresh extends the old backing in place with the units the
    /// last child gained, instead of copying every child's text again.
    pub(super) fn refresh_textarea(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        cause: TextareaRefresh,
    ) -> Result<(), SessionError> {
        let reset = cause == TextareaRefresh::Reset;
        let Some((slot, mut state)) = self.textarea_slot(control, node)? else {
            return Ok(());
        };
        let old = textarea_raw(&self.payloads, control, state.raw)?;
        let old_units = old.len();
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if old.block().holders() != 1 {
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
        let mut raw = match cause {
            TextareaRefresh::Append { added } if dispose_previous => RawValue::Extend(added),
            _ => RawValue::Copy(Utf16Buffer::new(control)?),
        };
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
                    && let ConstructionNodeKind::Text(value) = &self
                        .construction
                        .node_at(child)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                {
                    if let RawValue::Copy(raw) = &mut raw {
                        raw.extend_units(value.code_units(), control)?;
                    }
                    let added = value.len();
                    let end = units.checked_add(added).ok_or_else(|| {
                        control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                    })?;
                    let source = self.field_slot(control, StringLocation::Text(child))?;
                    let mut incoming = self.copied_slot_group(control, source)?;
                    if incoming.is_none() && self.graph_recorded() {
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
        let raw = match raw {
            RawValue::Copy(raw) => raw,
            RawValue::Extend(added) => {
                // The children and the raw value must agree on what was added.
                if units.checked_sub(old_units) != Some(added.get()) {
                    return Err(latch_registry_row(control));
                }
                return self.extend_textarea_raw(control, node, slot, state, group, added.get());
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

    /// Commits a refresh whose new raw value is the old one with the last
    /// `added` units of the last child at its end: the new row takes over the
    /// old row's backing, extended in place, and the old row keeps only its
    /// retired origins. Every step that can refuse, other than the extension
    /// itself, comes first, and a refused extension leaves the old row as it was.
    fn extend_textarea_raw(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        mut state: TextareaState,
        group: Option<OriginGroupRef>,
        added: usize,
    ) -> Result<(), SessionError> {
        let prepared = self.prepare_payload_slot_owned(control)?;
        let next = self
            .payloads
            .push_row(prepared, RetainedPayload::Retired)
            .map_err(|_| latch_registry_row(control))?;
        if let Some(group) = &group {
            self.evidence
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .bind_leaf_value(control, group, next)?;
        }
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<DocumentString>() as u64,
        )?;
        payload_write::<TextareaState>(control, 1)?;
        payload_write::<RetainedPayload>(control, 2)?;
        self.retire_field_group(control, state.raw)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let parent = self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?;
        let last = parent
            .child_count()
            .checked_sub(1)
            .and_then(|last| parent.child_at(last))
            .and_then(|child| self.construction.node_at(child.position()))
            .map(|record| record.kind());
        let Some(ConstructionNodeKind::Text(text)) = last else {
            return Err(latch_registry_row(control));
        };
        let Some(delta) = text
            .len()
            .checked_sub(added)
            .and_then(|start| text.code_units().get(start..))
        else {
            return Err(latch_registry_row(control));
        };
        let Some(row) = self.payloads.values.get_mut(state.raw) else {
            return Err(latch_registry_row(control));
        };
        let (value, mut allocation, origins, location) =
            match mem::replace(row, RetainedPayload::Retired) {
                RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location,
                } => (value, allocation, origins, location),
                other => {
                    *row = other;
                    return Err(latch_registry_row(control));
                }
            };
        let extended = match control.extend_managed_document_string(&mut allocation, value, delta) {
            Ok(extended) => extended,
            Err(refused) => {
                *row = RetainedPayload::String {
                    value: refused.value,
                    allocation,
                    origins,
                    location,
                };
                return Err(refused.stop.into());
            }
        };
        if let Some(origins) = origins {
            *row = RetainedPayload::HistoricalString { origins };
        }
        let Some(new_row) = self.payloads.values.get_mut(next) else {
            return Err(latch_registry_row(control));
        };
        *new_row = RetainedPayload::String {
            value: extended,
            allocation,
            origins: group,
            location: StringLocation::Detached,
        };
        let Some(RetainedPayload::Textarea { state: stored, .. }) =
            self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        state.raw = next;
        *stored = state;
        Ok(())
    }
}
