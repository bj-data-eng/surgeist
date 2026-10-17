//! HTML Dialog ordinary hooks against the one canonical Draft.
//!
//! No service is acknowledged here: the admitted native environment stops setup
//! at the actual inactive guard, and the sealed state proves existing services
//! are null. A reached unsupported action refuses before atomic publication.
use super::*;
use crate::dialog::{DocumentInteractiveState, DomDialogState};
use surgeist_html::{
    HtmlDialogAction as Action, HtmlDialogCursor, HtmlDialogOperation as Operation,
    HtmlDialogOutcome, HtmlDialogReply as Reply, KnownHtmlAttribute,
};

impl Draft {
    fn dialog_state(&self, key: Key) -> Result<&DomDialogState, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Err(DomError::DialogInapplicable);
        };
        element.dialog_state().ok_or(DomError::DialogInapplicable)
    }
    fn is_dialog(&self, key: Key) -> Result<bool, DomError> {
        Ok(matches!(&self.node(key)?.kind,
            DomNodeKind::Element(element) if element.dialog_state().is_some()))
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
        if let Some(formation) = &formation {
            formation.retain_bytes(size_of::<DomDialogState>(), cancel)?;
        }
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

    fn dialog_list(&self, document: Key, top_layer: bool) -> Result<&[Key], DomError> {
        match self.node(document)?.interactive_document.as_ref() {
            Some(DocumentInteractiveState::Known {
                open_dialogs,
                top_layer: layer,
            }) => Ok(if top_layer { layer } else { open_dialogs }),
            Some(DocumentInteractiveState::Unavailable) => {
                Err(DomError::DialogDocumentStateUnavailable)
            }
            None => Err(DomError::WrongNodeKind),
        }
    }
    fn dialog_list_contains(
        &self,
        document: Key,
        member: Key,
        top_layer: bool,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let list = self.dialog_list(document, top_layer)?;
        for candidate in list {
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
        top_layer: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut index = 0;
        loop {
            self.source_step(cancel)?;
            let len = self.dialog_list(document, top_layer)?.len();
            if index >= len {
                break;
            }
            self.source_step(cancel)?;
            let list = self.dialog_list(document, top_layer)?;
            if list[index] != member {
                index += 1;
                continue;
            }
            // Preadmit the bounded structural movement before Vec::remove.
            // No callback or reentry occurs between admission and that effect.
            for _ in index + 1..list.len() {
                self.source_step(cancel)?;
            }
            check_cancel(cancel)?;
            let Some(DocumentInteractiveState::Known {
                open_dialogs,
                top_layer: layer,
            }) = self.node_mut(document).interactive_document.as_mut()
            else {
                unreachable!("qualified Document lists")
            };
            if top_layer { layer } else { open_dialogs }.remove(index);
        }
        Ok(())
    }

    fn dialog_hook(
        &mut self,
        key: Key,
        operation: Operation,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !self.is_dialog(key)? {
            return Ok(());
        }
        let mut cursor = HtmlDialogCursor::new(operation);
        loop {
            self.source_step(cancel)?;
            let action = cursor.action();
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
                Action::ReadTopLayerContains => Reply::TopLayerContains(
                    self.dialog_list_contains(self.node(key)?.document, key, true, cancel)?,
                ),
                Action::RemoveOpenDialog => {
                    self.dialog_list_remove(self.node(key)?.document, key, false, cancel)?;
                    Reply::Applied
                }
                Action::RemoveTopLayerImmediately => {
                    self.dialog_list_remove(self.node(key)?.document, key, true, cancel)?;
                    Reply::Applied
                }
                // The closed service invariant establishes that the actual flag
                // is already false. This fulfills the source idempotent write;
                // it does not manufacture a hosted/modal transition.
                Action::ClearModal if !self.dialog_state(key)?.is_modal() => Reply::Applied,
                _ => return Err(DomError::DialogProcessingUnavailable { action }),
            };
            cursor.reply(reply).map_err(DomError::DialogProtocol)?;
        }
    }
    pub(super) fn dialog_attribute_change(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        old_present: bool,
        new_present: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !self.is_dialog(key)? {
            return Ok(());
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
            return Ok(());
        }
        self.source_step(cancel)?;
        if let Some(operation) = surgeist_html::dialog_attribute_operation(
            name.namespace(),
            KnownHtmlAttribute::Open,
            old_present,
            new_present,
        ) {
            self.dialog_hook(key, operation, cancel)?;
        }
        Ok(())
    }
    pub(super) fn dialog_insertion(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.dialog_hook(key, Operation::Inserted, cancel)
    }
    pub(super) fn dialog_removal(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        self.dialog_hook(key, Operation::Removed, cancel)
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
                crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096);
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
                byte_limit, unit_limit, 10_000_000, 16, 4096, 4096,
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
        let limits = crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096);
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
                crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096);
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
                16_000_000, 100_000, work, 16, 4096, 4096,
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
