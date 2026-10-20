//! A real excess-capacity output supplied at the private allocation boundary.
//! Input spare capacity cannot establish the mapped output's retained extent.
use super::*;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

enum Output {
    Ready(Vec<FormPastName<NodeId>>),
    Taken,
}
thread_local! { static OUTPUT: RefCell<Option<Output>> = const { RefCell::new(None) }; }

pub(in crate::store) struct SuppliedOutput(PhantomData<Rc<()>>);
impl SuppliedOutput {
    pub fn begin(output: Vec<FormPastName<NodeId>>) -> Self {
        assert!(output.is_empty());
        OUTPUT.with(|slot| {
            let mut slot = slot.borrow_mut();
            assert!(slot.is_none(), "one supplied mapped output per thread");
            *slot = Some(Output::Ready(output));
        });
        Self(PhantomData)
    }
    pub fn taken(&self) -> bool {
        OUTPUT.with(|slot| matches!(*slot.borrow(), Some(Output::Taken)))
    }
}
impl Drop for SuppliedOutput {
    fn drop(&mut self) {
        OUTPUT.with(|slot| *slot.borrow_mut() = None);
    }
}
pub(super) fn output_buffer(fallback: Vec<FormPastName<NodeId>>) -> Vec<FormPastName<NodeId>> {
    OUTPUT.with(|slot| {
        let mut slot = slot.borrow_mut();
        match slot.as_mut() {
            Some(output @ Output::Ready(_)) => {
                let Output::Ready(buffer) = std::mem::replace(output, Output::Taken) else {
                    unreachable!()
                };
                buffer
            }
            Some(Output::Taken) | None => fallback,
        }
    })
}

fn raw_extent(entries: &[FormPastName<Key>], capacity: usize) -> custody::Usage {
    let mut bytes = size_of::<payload::Payload<History<NodeId>>>()
        + capacity * size_of::<FormPastName<NodeId>>();
    let mut units = 0;
    for entry in entries {
        let capacity = entry.name.shared_code_units().capacity();
        bytes += size_of::<Vec<u16>>() + capacity * size_of::<u16>();
        units += capacity;
    }
    custody::Usage::new(bytes, units)
}

#[test]
fn mapped_output_excess_has_an_exact_source_body_gate_and_sealed_receipt() {
    let dom = Dom::new(
        DomLimits::new(
            32,
            4096,
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
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    let mut raw_name = Vec::with_capacity(17);
    raw_name.extend([0xd800, 0, 0xdc00]);
    let name = DocumentString::from_utf16(raw_name);
    let entries = vec![
        FormPastName {
            name: name.clone(),
            target: Key::Created(0),
        },
        FormPastName {
            name,
            target: Key::Created(1),
        },
    ];
    let names = Names::staged(entries, None, &draft, &cancel).unwrap();
    let NamesBacking::Staged(input) = &names.backing else {
        unreachable!()
    };
    let mapped = NodeId {
        owner: draft.store,
        slot: 7,
        generation: 3,
    };
    draft.custody_registry().reconcile_releases();
    let before = draft.custody_ledger().total();
    let original_limits = draft.source_limits.unwrap();
    for short in [true, false] {
        let buffer = Vec::with_capacity(11);
        let capacity = buffer.capacity();
        assert!(capacity > input.entries().len());
        let expected = raw_extent(input.entries(), capacity);
        let supplied = SuppliedOutput::begin(buffer);
        let boundary = before.bytes() + expected.bytes() + draft.source_other_bytes;
        draft.source_limits.as_mut().unwrap().retained_bytes = boundary - usize::from(short);
        let prepared = names.prepare_projection(&draft, &|_| mapped, &cancel);
        assert!(supplied.taken());
        drop(supplied);
        if short {
            assert!(matches!(
                prepared,
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit
                }) if limit == boundary - 1
            ));
            draft.custody_registry().reconcile_releases();
            assert_eq!(draft.custody_ledger().total(), before);
            continue;
        }
        let prepared = prepared.unwrap();
        let PreparedNames::Pending(pending) = &prepared else {
            panic!("a mapped nonempty history is newly materialized")
        };
        assert_eq!(pending.value().value.entries.capacity(), capacity);
        assert_eq!(pending.value().own_usage, expected);
        assert_eq!(
            draft.custody_ledger().total(),
            before.checked_add(expected).unwrap()
        );
        // The exact boundary above is the output body gate. Registry metadata
        // has its own real admission and is not priced as the body allocation.
        draft.source_limits = Some(original_limits);
        let formed = prepared.seal(&draft, &cancel).unwrap();
        assert_eq!(formed.value.capacity(), capacity);
        assert_eq!(formed.value.own_usage(), expected);
        assert_eq!(
            formed._custody.as_ref().unwrap().inspect_payload_usage(),
            expected
        );
        assert_eq!(
            formed
                .value
                .entries()
                .iter()
                .map(|entry| entry.target)
                .collect::<Vec<_>>(),
            [mapped, mapped]
        );
        drop(formed);
        draft.custody_registry().reconcile_releases();
        assert_eq!(draft.custody_ledger().total(), before);

        // Before registry sealing, this actual Form Pending contributes only
        // its affine body receipt. Its destructor must still see the complete
        // excess-capacity extent, not an unrelated surviving owner's charge.
        let output = Vec::with_capacity(capacity);
        let release_extent = raw_extent(input.entries(), output.capacity());
        let supplied = SuppliedOutput::begin(output);
        let prepared = names
            .prepare_projection(&draft, &|_| mapped, &cancel)
            .unwrap();
        assert!(supplied.taken());
        drop(supplied);
        let PreparedNames::Pending(pending) = &prepared else {
            unreachable!()
        };
        let destruction = payload::destruction_observation::Observation::begin(
            pending.value().identity,
            draft.custody_ledger(),
        );
        drop(prepared);
        assert_eq!(
            destruction.observed(),
            Some(before.checked_add(release_extent).unwrap())
        );
        assert_eq!(draft.custody_ledger().total(), before);
    }
    draft.source_limits = Some(original_limits);
    drop(names);
}
