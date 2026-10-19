use super::*;
use crate::store::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn unlimited(_: Usage) -> Result<(), DomError> {
    Ok(())
}

fn text_extent(capacity: usize) -> Usage {
    Usage::new(
        size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        capacity,
    )
}

#[test]
fn leaf_group_identity_refusal_precedes_factory_and_all_new_admission() {
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
    let baseline = ledger.total();
    let called = std::cell::Cell::new(false);
    let refusal = identity_refusal::Refusal::begin();
    let result = form_group(
        &registry,
        None,
        source::storage::Payload::default(),
        Usage::new(size_of::<Payload<[u64; 3]>>(), 0),
        &mut |_| panic!("essential identity precedes retention"),
        || {
            called.set(true);
            Ok([11u64, 13, 17])
        },
    );
    assert!(matches!(result, Err(DomError::RepresentationExhausted)));
    assert!(refusal.reached());
    assert!(!called.get());
    assert_eq!(ledger.total(), baseline);
}

struct OriginalText {
    value: DocumentString,
    ledger: custody::Ledger,
    during_drop: Arc<std::sync::Mutex<Option<Usage>>>,
}
impl Drop for OriginalText {
    fn drop(&mut self) {
        *self.during_drop.lock().unwrap() = Some(self.ledger.total());
    }
}

#[test]
fn source_leaf_factory_transfers_original_text_once_and_refuses_before_build_at_one_short() {
    let dom = Dom::new(
        DomLimits::new(
            8,
            4096,
            16,
            0,
            0,
            8,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let (draft, _) = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap();
    let ledger = draft.custody_ledger();
    let registry = draft.custody_registry();
    for short in [true, false] {
        registry.reconcile_releases();
        let budget = source::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let before = ledger.total();
        let original = text_extent(17);
        budget
            .retain_payload(
                source::storage::Payload {
                    bytes: original.bytes(),
                    units: original.units(),
                },
                &cancel,
            )
            .unwrap();
        let mut text = Vec::with_capacity(17);
        let actual = text_extent(text.capacity());
        let excess = actual.checked_sub(original).unwrap();
        budget
            .retain_payload(
                source::storage::Payload {
                    bytes: excess.bytes(),
                    units: excess.units(),
                },
                &cancel,
            )
            .unwrap();
        text.extend([0xd800, 0, 0xdc00, 31]);
        let during_drop = Arc::new(std::sync::Mutex::new(None));
        let input = OriginalText {
            value: DocumentString::from_utf16(text),
            ledger: ledger.clone(),
            during_drop: during_drop.clone(),
        };
        let weak = Arc::downgrade(input.value.shared_code_units());
        let called = std::cell::Cell::new(false);
        let own = Usage::new(size_of::<Payload<OriginalText>>(), 0)
            .checked_add(actual)
            .unwrap();
        // This is the pre-factory body gate. Sealing's separately admitted
        // controls are deliberately outside this raw exact/one-short amount.
        let boundary = before.bytes() + own.bytes() - usize::from(short);
        let spent = draft.source_work.load(Ordering::Relaxed);
        let result = form_group(
            registry,
            Some(&budget),
            source::storage::Payload {
                bytes: actual.bytes(),
                units: actual.units(),
            },
            own,
            &mut |next| {
                draft.admit_custody_usage(next, &cancel)?;
                if !called.get() && next.bytes() > boundary {
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: boundary,
                    })
                } else {
                    Ok(())
                }
            },
            || {
                called.set(true);
                assert_eq!(
                    ledger.total(),
                    before.checked_add(own).unwrap(),
                    "the original text is transferred, never retained twice"
                );
                assert_eq!(input.value.code_units(), [0xd800, 0, 0xdc00, 31]);
                Ok(input)
            },
        );
        assert!(draft.source_work.load(Ordering::Relaxed) > spent);
        assert_eq!(budget.owned_payload().bytes, 0);
        assert_eq!(budget.owned_payload().units, 0);
        if short {
            assert!(
                matches!(result, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == boundary)
            );
            assert!(!called.get());
            assert_eq!(
                *during_drop.lock().unwrap(),
                Some(before.checked_add(actual).unwrap())
            );
        } else {
            let sealed = result.unwrap();
            assert!(called.get());
            assert_eq!(sealed.custody.inspect_payload_usage(), own);
            assert_eq!(
                sealed.value.value.value.code_units(),
                [0xd800, 0, 0xdc00, 31]
            );
            assert!(during_drop.lock().unwrap().is_none());
            drop(sealed);
            assert!(during_drop.lock().unwrap().unwrap().bytes() >= before.bytes() + own.bytes());
        }
        assert!(weak.upgrade().is_none());
        registry.reconcile_releases();
        assert_eq!(ledger.total(), before);
        drop(budget);
    }
}
