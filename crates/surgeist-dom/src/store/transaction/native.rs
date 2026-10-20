//! Selected native feature edits. Only the reached native body is copied; its
//! immutable old target remains owned by the original Element and snapshots.
use super::super::*;
use crate::element::{ElementChildren, FormedElement};
use std::ops::{Deref, DerefMut};

type Getter<T> = for<'a> fn(&'a DomElement) -> Option<&'a Arc<payload::Payload<T>>>;
type Setter<T> = fn(&mut DomElement, Arc<payload::Payload<T>>);
type Measure<T> = fn(&T) -> Result<custody::Usage, DomError>;
struct Access<T> {
    get: Getter<T>,
    set: Setter<T>,
    measure: Measure<T>,
}

/// One selected, admitted mutable native body. The Box is its actual private
/// holder; the reserved final target extent covers that smaller holder too.
/// It is released before the same value moves into its immutable target.
pub(in crate::store) struct NativeMut<'a, T> {
    draft: &'a mut Draft,
    key: Key,
    candidate: Option<custody::Held<Box<T>>>,
    reserved: Option<payload::Reserved<T>>,
    set: Setter<T>,
    measure: Measure<T>,
}
impl<T> Deref for NativeMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.candidate
            .as_ref()
            .expect("selected native candidate")
            .value()
    }
}
impl<T> DerefMut for NativeMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.candidate
            .as_mut()
            .expect("selected native candidate")
            .value_mut()
    }
}

fn reconcile(
    debit: &mut custody::Debit,
    actual: custody::Usage,
    admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let before = debit.usage();
    let extra = custody::Usage::new(
        actual.bytes().saturating_sub(before.bytes()),
        actual.units().saturating_sub(before.units()),
    );
    if extra != custody::Usage::ZERO {
        debit.grow(extra, admit)?;
    }
    // Mutating the selected value has already destroyed the replaced field.
    // Release only its now-unused occurrence, after admitting any new extent.
    let excess = debit.usage().checked_sub(actual)?;
    drop(debit.split(excess)?);
    Ok(())
}

impl<T> NativeMut<'_, T> {
    pub fn commit_charged(
        mut self,
        charge: Option<source::checkpoint::Charge>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(debit) = charge.and_then(source::checkpoint::Charge::into_debit) {
            self.candidate
                .as_mut()
                .expect("selected native candidate")
                .debit_mut()
                .absorb(debit)
                .expect("original native field ledger");
        }
        self.commit(cancel)
    }
    pub fn commit_formed(
        mut self,
        formation: Option<source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(debit) = formation.and_then(source::formation::Formation::into_debit) {
            self.candidate
                .as_mut()
                .expect("selected native candidate")
                .debit_mut()
                .absorb(debit)
                .expect("original native field ledger");
        }
        self.commit(cancel)
    }
    pub fn commit(mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let mut candidate = self.candidate.take().expect("one native commit");
        let usage = (self.measure)(candidate.value())?;
        reconcile(candidate.debit_mut(), usage, |next| {
            self.draft.admit_custody_usage(next, cancel)
        })?;
        let (value, debit) = candidate.into_parts();
        let value = *value; // deallocate the private holder before Arc allocation
        let pending = self
            .reserved
            .take()
            .expect("one native identity")
            .materialize(custody::Held::new(value, debit), usage, |next| {
                self.draft.admit_custody_usage(next, cancel)
            })?;
        let native = pending.seal(self.draft.custody_registry(), |next| {
            self.draft.admit_custody_usage(next, cancel)
        })?;
        self.draft
            .install_native(self.key, native, self.set, cancel)
    }
}

impl Draft {
    fn native_mut<T: Clone>(
        &mut self,
        key: Key,
        access: Access<T>,
        cancel: &AtomicBool,
    ) -> Result<NativeMut<'_, T>, DomError> {
        self.materialize_node(key, cancel)?;
        let reserved = payload::Reserved::new()?;
        let old = (access.get)(
            self.node(key)?
                .element()
                .ok_or(DomError::WrongNodeKind)?
                .language(),
        )
        .ok_or(DomError::WrongNodeKind)?;
        let usage = (access.measure)(&old.value)?;
        let debit = self
            .custody_ledger()
            .retain(usage, |next| self.admit_custody_usage(next, cancel))?;
        let candidate = custody::Held::new(Box::new(old.value.clone()), debit);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::RecordMaterialized {
                bytes: size_of::<T>(),
            });
        }
        Ok(NativeMut {
            draft: self,
            key,
            candidate: Some(candidate),
            reserved: Some(reserved),
            set: access.set,
            measure: access.measure,
        })
    }

    fn install_native<T>(
        &mut self,
        key: Key,
        native: payload::Sealed<T>,
        set: Setter<T>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut value = self
            .node(key)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .language()
            .clone();
        set(&mut value, native.value);
        let children = ElementChildren {
            native: Some(native.custody),
            ..ElementChildren::default()
        };
        self.replace_element_formed(key, FormedElement { value, children }, None, cancel)
    }

    fn replace_native_formed<T>(
        &mut self,
        key: Key,
        value: T,
        formation: Option<source::formation::Formation>,
        access: Access<T>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Retain the actual incoming data before its original control through
        // all preparatory failures. Never clone a discarded old native state.
        struct Incoming<T> {
            value: T,
            formation: Option<source::formation::Formation>,
        }
        let incoming = Incoming { value, formation };
        (access.get)(
            self.node(key)?
                .element()
                .ok_or(DomError::WrongNodeKind)?
                .language(),
        )
        .ok_or(DomError::WrongNodeKind)?;
        self.materialize_node(key, cancel)?;
        let reserved = payload::Reserved::new()?;
        let usage = (access.measure)(&incoming.value)?;
        let Incoming { value, formation } = incoming;
        let debit = formation
            .and_then(source::formation::Formation::into_debit)
            .map_or_else(
                || {
                    self.custody_ledger().retain(custody::Usage::ZERO, |next| {
                        self.admit_custody_usage(next, cancel)
                    })
                },
                Ok,
            )?;
        let mut incoming = custody::Held::new(value, debit);
        #[cfg(test)]
        super::queues::admission_observation::before_native_target(self, key);
        reconcile(incoming.debit_mut(), usage, |next| {
            self.admit_custody_usage(next, cancel)
        })?;
        let pending = reserved.materialize(incoming, usage, |next| {
            self.admit_custody_usage(next, cancel)
        })?;
        #[cfg(test)]
        super::queues::admission_observation::after_native_target(self, key);
        let native = pending.seal(self.custody_registry(), |next| {
            self.admit_custody_usage(next, cancel)
        })?;
        self.install_native(key, native, access.set, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Dom, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                64,
                0,
                0,
                16,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let dialog = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let mut raw = Vec::with_capacity(73);
        raw.extend_from_slice(&[0x41, 0xd800, 0x5a]);
        batch.set_dialog_return_value(&dialog, DocumentString::from_utf16(raw));
        let commit = dom.apply(batch, &cancel).unwrap();
        let dialog = commit.created(&dialog).unwrap().unwrap();
        (dom, dialog)
    }

    /// Independently enumerate only this fixture's direct state. This is not
    /// the production helper or a cached Payload receipt.
    fn raw_dialog(state: &crate::dialog::DomDialogState) -> custody::Usage {
        let mut bytes = size_of::<payload::Payload<crate::dialog::DomDialogState>>();
        let mut units = 0;
        for text in [
            Some(state.return_value()),
            state.request_return_value.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            let capacity = text.shared_code_units().capacity();
            bytes += size_of::<Vec<u16>>() + capacity * size_of::<u16>();
            units += capacity;
        }
        custody::Usage::new(bytes, units)
    }

    #[test]
    fn selected_native_clone_admits_full_raw_overlap_and_refuses_one_short() {
        let cancel = AtomicBool::new(false);
        for resource in [
            DomResource::SourceRetainedBytes,
            DomResource::SourceRetainedUnits,
        ] {
            for short in [false, true] {
                let (dom, dialog) = fixture();
                let before = dom.snapshot();
                let batch = DomBatch::new();
                let limits = crate::DomSourceLimits::new(
                    16_000_000, 1_000_000, 1_000_000, 32, 4096, 4096, 1,
                );
                let (mut draft, _) =
                    Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel)
                        .unwrap();
                let key = draft.resolve(dialog.into()).unwrap();
                draft.materialize_node(key, &cancel).unwrap();
                let old = draft
                    .element(key)
                    .unwrap()
                    .language()
                    .dialog_payload()
                    .unwrap()
                    .clone();
                let expected = raw_dialog(&old.value);
                let baseline = draft.custody_ledger().total();
                let mut policy = limits;
                let limit = match resource {
                    DomResource::SourceRetainedBytes => {
                        let limit = baseline.bytes() + draft.source_other_bytes + expected.bytes()
                            - usize::from(short);
                        policy.retained_bytes = limit;
                        limit
                    }
                    DomResource::SourceRetainedUnits => {
                        let limit = baseline.units() + draft.source_other_units + expected.units()
                            - usize::from(short);
                        policy.retained_units = limit;
                        limit
                    }
                    _ => unreachable!(),
                };
                draft.source_limits = Some(policy);
                match draft.dialog_state_mut(key, &cancel) {
                    Err(error) => {
                        assert!(short);
                        assert_eq!(error, DomError::ResourceLimit { resource, limit });
                    }
                    Ok(guard) => {
                        assert!(!short);
                        assert_eq!(guard.return_value().code_units(), [0x41, 0xd800, 0x5a]);
                        assert!(Arc::ptr_eq(
                            guard.return_value().shared_code_units(),
                            old.value.return_value().shared_code_units()
                        ));
                        assert_eq!(
                            guard.draft.custody_ledger().total(),
                            baseline.checked_add(expected).unwrap()
                        );
                        drop(guard);
                    }
                }
                assert_eq!(draft.custody_ledger().total(), baseline);
                assert!(Arc::ptr_eq(
                    draft
                        .element(key)
                        .unwrap()
                        .language()
                        .dialog_payload()
                        .unwrap(),
                    &old
                ));
                assert_eq!(dom.snapshot().stamp(), before.stamp());
            }
        }
    }

    #[test]
    fn committed_native_target_keeps_old_snapshot_and_releases_after_last_semantic_owner() {
        let cancel = AtomicBool::new(false);
        let (dom, dialog) = fixture();
        let before = dom.snapshot();
        let batch = DomBatch::new();
        let mut draft = Draft::new(&dom, batch.owner.clone(), &cancel).unwrap();
        let key = draft.resolve(dialog.into()).unwrap();
        let old = draft
            .element(key)
            .unwrap()
            .language()
            .dialog_payload()
            .unwrap()
            .clone();
        let expected = raw_dialog(&old.value);
        let mut guard = draft.dialog_state_mut(key, &cancel).unwrap();
        guard.force_watcher_enabled = true;
        guard.commit(&cancel).unwrap();
        let next = draft
            .element(key)
            .unwrap()
            .language()
            .dialog_payload()
            .unwrap()
            .clone();
        assert_ne!(old.identity, next.identity);
        assert!(!Arc::ptr_eq(&old, &next));
        assert!(!old.value.force_watcher_enabled);
        assert!(next.value.force_watcher_enabled);
        assert_eq!(
            std::mem::size_of_val(&*next),
            size_of::<payload::Payload<crate::dialog::DomDialogState>>()
        );
        assert_eq!(next.own_usage, expected);
        assert_eq!(next.value.return_value().code_units(), [0x41, 0xd800, 0x5a]);
        assert!(Arc::ptr_eq(
            old.value.return_value().shared_code_units(),
            next.value.return_value().shared_code_units()
        ));
        let weak = Arc::downgrade(&next);
        drop(draft);
        assert!(weak.upgrade().is_some());
        assert_eq!(
            before
                .node(dialog)
                .unwrap()
                .as_element()
                .unwrap()
                .dialog_state()
                .unwrap()
                .return_value()
                .code_units(),
            [0x41, 0xd800, 0x5a]
        );
        drop(next);
        assert!(weak.upgrade().is_none());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
}

macro_rules! native_edit {
    ($edit:ident, $get:ident, $set:ident, $measure:ident, $state:ty) => {
        impl Draft {
            pub(in crate::store) fn $edit(
                &mut self,
                key: Key,
                cancel: &AtomicBool,
            ) -> Result<NativeMut<'_, $state>, DomError> {
                self.native_mut(
                    key,
                    Access {
                        get: DomElement::$get,
                        set: DomElement::$set,
                        measure: source::storage::$measure,
                    },
                    cancel,
                )
            }
        }
    };
}
native_edit!(
    input_state_mut,
    input_payload,
    set_input_payload,
    input_state,
    crate::DomInputState
);
native_edit!(
    textarea_state_mut,
    textarea_payload,
    set_textarea_payload,
    textarea_state,
    crate::DomTextareaState
);
native_edit!(
    output_state_mut,
    output_payload,
    set_output_payload,
    output_state,
    crate::DomOutputState
);
native_edit!(
    select_state_mut,
    select_payload,
    set_select_payload,
    select_state,
    crate::DomSelectState
);
native_edit!(
    fieldset_state_mut,
    fieldset_payload,
    set_fieldset_payload,
    fieldset_state,
    crate::DomFieldsetState
);
native_edit!(
    button_state_mut,
    button_payload,
    set_button_payload,
    button_state,
    crate::DomButtonState
);
native_edit!(
    dialog_state_mut,
    dialog_payload,
    set_dialog_payload,
    dialog_state,
    crate::dialog::DomDialogState
);
native_edit!(
    style_state_mut,
    style_payload,
    set_style_payload,
    style_state,
    crate::DomStyleState
);

impl Draft {
    /// Consume a complete preformed Dialog candidate. Its original Formation
    /// includes the raw state body and retained operand occurrences; only the
    /// actual immutable target's residual header is acquired before allocation.
    pub(in crate::store) fn replace_dialog_state_formed(
        &mut self,
        key: Key,
        value: crate::dialog::DomDialogState,
        formation: Option<source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.replace_native_formed(
            key,
            value,
            formation,
            Access {
                get: DomElement::dialog_payload,
                set: DomElement::set_dialog_payload,
                measure: source::storage::dialog_state,
            },
            cancel,
        )
    }
}
