use super::*;
use crate::accounting::{
    RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderClock,
    RenderCpuScope, RenderLimits, RenderPhase, RenderProfile, RenderQuantity, RenderWork,
    RendererCpu,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use surgeist_ledger::Dimension;

fn failure<T>(result: Result<T>) -> crate::Error {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected a checked refusal"),
    }
}
fn contract(error: &crate::Error) -> StorageContractError {
    assert!(error.accounting_diagnostic().is_none());
    *std::error::Error::source(error)
        .and_then(|source| source.downcast_ref())
        .expect("typed storage source")
}
fn run(
    options: RenderAccountingOptions,
    action: impl FnOnce(&Account) -> Result<()>,
) -> (Result<()>, RenderAccountingContext) {
    let mut context = RenderAccountingContext::new(options);
    let result = context.run(
        RenderAttemptKind::GlyphRun,
        RenderPhase::Preparation,
        action,
    );
    (result, context)
}

#[test]
fn append_insert_and_bound_copy_have_literal_contents() {
    let account = Account::for_test();
    let mut values = AccountedVec::new(&account, 6, RenderResource::ScratchCpuBytes).unwrap();
    values.append_slot().unwrap().write(10_u8);
    values.append_slot().unwrap().write(30);
    let insert = values.insert_slot(1).unwrap();
    assert_eq!(insert.movement_count(), 2); // suffix30 and the inserted20
    insert.write(20);
    let source = [40, 50];
    let copy = values.copy_slot(&source).unwrap();
    assert_eq!(copy.element_count(), 2);
    copy.write();
    assert_eq!(values.as_slice(), [10, 20, 30, 40, 50]);
}

#[test]
fn capacity_and_insertion_failures_are_consumer_contracts_not_ledger_defects() {
    let account = Account::for_test();
    let mut values = AccountedVec::<u8>::new(&account, 0, RenderResource::ScratchCpuBytes).unwrap();
    assert_eq!(
        contract(&failure(values.append_slot())),
        StorageContractError::ReservedCapacity {
            needed: 1,
            capacity: 0
        }
    );
    assert_eq!(
        contract(&failure(values.insert_slot(1))),
        StorageContractError::InvalidInsertion { index: 1, len: 0 }
    );
    assert_eq!(
        contract(&failure(values.insert_slot(0))),
        StorageContractError::ReservedCapacity {
            needed: 1,
            capacity: 0
        }
    );
    assert_eq!(
        contract(&failure(values.copy_slot(&[7, 8]))),
        StorageContractError::ReservedCapacity {
            needed: 2,
            capacity: 0
        }
    );
    assert!(values.is_empty());
}

#[test]
fn abandoned_slots_and_empty_copy_do_not_invent_work_or_growth() {
    let (result, context) = run(RenderAccountingOptions::default(), |account| {
        let mut values = account.temporary::<u8>(2)?;
        let capacity = values.capacity();
        {
            let _unused = values.append_slot()?;
        }
        {
            let _unused = values.insert_slot(0)?;
        }
        values.copy_slot(&[])?.write();
        assert!(values.is_empty());
        assert_eq!(values.capacity(), capacity);
        Ok(())
    });
    result.unwrap();
    for &work in RenderWork::ALL {
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(work),
            0,
            "{work:?}"
        );
    }
}

#[test]
fn zero_sized_logical_length_overflow_refuses_before_write() {
    // A real ZST Vec can have this logical length with no allocation or loop.
    // This is the standalone preparation owner, retaining its Glyph error domain.
    let (result, context) = run(RenderAccountingOptions::default(), |account| {
        let hold = account.acquire(RenderResource::ScratchCpuBytes, 0)?;
        let mut values = AccountedVec::from_parts(vec![(); usize::MAX], hold)?;
        assert!(matches!(
            failure(values.append_slot()).glyph_preparation_diagnostic(),
            Some(crate::GlyphPreparationError::CapacityOverflow)
        ));
        assert!(matches!(
            failure(values.insert_slot(0)).glyph_preparation_diagnostic(),
            Some(crate::GlyphPreparationError::CapacityOverflow)
        ));
        values.copy_slot(&[])?.write();
        assert_eq!(values.len(), usize::MAX);
        Ok(())
    });
    result.unwrap();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
}

#[test]
fn replacement_counts_existing_length_once_and_admits_actual_overlap() {
    let mut actual_peak = 0;
    let (result, context) = run(RenderAccountingOptions::default(), |account| {
        let mut values = account.temporary::<u8>(8)?;
        values.copy_slot(&[11, 22])?.write();
        let old_capacity = values.capacity();
        values.grow(1)?; // len2+1 already fits: no relocation
        values.grow(7)?; // len2+7 exceeds8: exactly two existing elements move
        actual_peak = old_capacity + values.capacity();
        assert_eq!(values.as_slice(), [11, 22]);
        Ok(())
    });
    result.unwrap();
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::StorageRelocations), 2);
    assert_eq!(
        usage.peak(RenderResource::ScratchCpuBytes),
        actual_peak as u64
    );
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn one_short_relocation_cap_preserves_a_real_accepted_prefix_and_all_old_values() {
    let options = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(RenderWork::StorageRelocations, 2)
            .unwrap(),
    );
    let (result, context) = run(options, |account| {
        let mut prefix = account.temporary::<u8>(1)?;
        prefix.append_slot()?.write(7);
        prefix.grow(1)?; // one actual move accepted
        let mut refused = account.temporary::<u8>(2)?;
        refused.copy_slot(&[8, 9])?.write();
        let capacity = refused.capacity();
        let error = failure(refused.grow(1)); // requests two more moves; total3 > cap2
        assert_eq!(refused.as_slice(), [8, 9]);
        assert_eq!(refused.capacity(), capacity);
        let Some(crate::RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
        else {
            panic!("original relocation refusal")
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::StorageRelocations)
        );
        assert_eq!((refusal.current(), refusal.amount()), (1, 2));
        Err(error)
    });
    assert!(result.is_err());
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::StorageRelocations), 1);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    let (result, retry) = run(RenderAccountingOptions::default(), |account| {
        let mut prefix = account.temporary::<u8>(1)?;
        prefix.append_slot()?.write(7);
        prefix.grow(1)?;
        let mut values = account.temporary::<u8>(2)?;
        values.copy_slot(&[8, 9])?.write();
        values.grow(1)?;
        assert_eq!(values.as_slice(), [8, 9]);
        Ok(())
    });
    result.unwrap();
    assert_eq!(
        retry
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::StorageRelocations),
        3
    );
}

#[test]
fn callback_failure_precedes_central_relocation_and_preserves_its_exact_domain() {
    let (result, context) = run(RenderAccountingOptions::default(), |account| {
        let mut values = account.temporary::<u8>(2)?;
        values.copy_slot(&[3, 4])?.write();
        let capacity = values.capacity();
        let error = failure(values.grow_with(1, |count| {
            assert_eq!(count, 2);
            Err(crate::GlyphPreparationError::CapacityOverflow.into())
        }));
        assert!(matches!(
            error.glyph_preparation_diagnostic(),
            Some(crate::GlyphPreparationError::CapacityOverflow)
        ));
        assert_eq!(values.as_slice(), [3, 4]);
        assert_eq!(values.capacity(), capacity);
        Err(error)
    });
    assert!(result.is_err());
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::StorageRelocations), 0);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn raw_native_backing_growth_uses_the_same_single_relocation_authority() {
    let (result, context) = run(RenderAccountingOptions::default(), |account| {
        let storage = account.storage();
        let mut values = Vec::<u8>::new();
        let mut hold = None;
        reserve_backing_with(
            &mut values,
            &mut hold,
            2,
            &storage,
            RenderResource::ScratchCpuBytes,
            |_| Ok(()),
        )?;
        values.extend_from_slice(&[5, 6]);
        reserve_backing_with(
            &mut values,
            &mut hold,
            3,
            &storage,
            RenderResource::ScratchCpuBytes,
            |count| {
                assert_eq!(count, 2);
                Ok(())
            },
        )?;
        assert_eq!(values, [5, 6]);
        let Some(hold) = hold else {
            panic!("admitted native backing")
        };
        hold.dispose(|| drop(values));
        Ok(())
    });
    result.unwrap();
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::StorageRelocations), 2);
    assert_eq!(usage.work(RenderWork::EncodingStreamCopies), 0);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn producer_keeps_owner_intact_and_original_callback_failure_then_retries() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let baseline = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let mut values =
        AccountedVec::<u8>::new_in(&storage, 4, RenderResource::RetainedCpuBytes).unwrap();
    let admitted = values.capacity() as u64;
    let error = failure(values.with_producer(2, |output| {
        output.push(7);
        Err::<(), _>(crate::GlyphPreparationError::CapacityOverflow.into())
    }));
    assert!(matches!(
        error.glyph_preparation_diagnostic(),
        Some(crate::GlyphPreparationError::CapacityOverflow)
    ));
    assert_eq!(values.as_slice(), [7]);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        baseline + admitted
    );
    let error = failure(values.with_producer(2, |output| {
        output.clear();
        output.extend_from_slice(&[1, 2, 3]);
        Ok(())
    }));
    assert_eq!(
        contract(&error),
        StorageContractError::ProducerLengthChanged {
            expected: 2,
            actual: 3
        }
    );
    values
        .with_producer(2, |output| {
            output.clear();
            output.extend_from_slice(&[8, 9]);
            Ok(())
        })
        .unwrap();
    assert_eq!(values.as_slice(), [8, 9]);
    drop(values);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        baseline
    );
}

#[test]
fn renderer_relocation_is_cpu_lifetime_work_not_forward_frame_work() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let opening = cpu
        .snapshot()
        .unwrap()
        .usage()
        .work(RenderWork::StorageRelocations);
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Encode, |_| {
            let mut values =
                AccountedVec::<u8>::new_in(&storage, 2, RenderResource::RetainedCpuBytes)?;
            values.copy_slot(&[1, 2])?.write();
            values.grow(1)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::StorageRelocations),
        0
    );
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::StorageRelocations)
            - opening,
        2
    );
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::StorageRelocations, 0)
        .unwrap();
    assert_eq!(
        limits.validate(RenderAttemptKind::Frame),
        Err(crate::RenderAccountingError::InvalidLimit(
            RenderWork::StorageRelocations
        ))
    );
    assert_eq!(
        limits.validate(RenderAttemptKind::Readback),
        Err(crate::RenderAccountingError::InvalidLimit(
            RenderWork::StorageRelocations
        ))
    );
}

static DISPOSAL_TIME: AtomicU64 = AtomicU64::new(0);
fn disposal_clock() -> std::time::Duration {
    std::time::Duration::from_millis(DISPOSAL_TIME.load(Ordering::SeqCst))
}
struct PayloadProbe {
    bytes: Vec<u8>,
    cpu: RendererCpu,
    admitted: u64,
    observed: Arc<AtomicU64>,
    drops: Arc<AtomicUsize>,
}
impl Drop for PayloadProbe {
    fn drop(&mut self) {
        self.observed.store(
            self.cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            Ordering::SeqCst,
        );
        assert_eq!(self.observed.load(Ordering::SeqCst), self.admitted);
        DISPOSAL_TIME.fetch_add(7, Ordering::SeqCst);
        self.drops.fetch_add(1, Ordering::SeqCst);
        assert!(self.bytes.capacity() >= self.bytes.len());
    }
}
#[test]
fn wrong_renderer_lifetime_keeps_paired_payload_credit_and_real_disposal_timing() {
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        for vector in [false, true] {
            DISPOSAL_TIME.store(0, Ordering::SeqCst);
            let cpu = RendererCpu::new(
                RenderAccountingOptions::default()
                    .with_profile(profile)
                    .with_clock(RenderClock::Host(disposal_clock)),
            )
            .unwrap();
            let baseline = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
            let observed = Arc::new(AtomicU64::new(0));
            let drops = Arc::new(AtomicUsize::new(0));
            let error = if vector {
                let mut values = AccountedVec::<PayloadProbe>::new_in(
                    &storage,
                    1,
                    RenderResource::RetainedCpuBytes,
                )
                .unwrap();
                let admitted =
                    baseline + (values.capacity() * std::mem::size_of::<PayloadProbe>()) as u64;
                values.append_slot().unwrap().write(PayloadProbe {
                    bytes: Vec::new(),
                    cpu: cpu.clone(),
                    admitted,
                    observed: observed.clone(),
                    drops: drops.clone(),
                });
                DISPOSAL_TIME.store(5, Ordering::SeqCst);
                failure(values.into_preparation())
            } else {
                let mut bytes =
                    AccountedVec::<u8>::new_in(&storage, 3, RenderResource::RetainedCpuBytes)
                        .unwrap();
                bytes.copy_slot(&[1, 2, 3]).unwrap().write();
                let admitted = baseline + bytes.capacity() as u64;
                let (bytes, hold) = bytes.into_parts().unwrap();
                let guarded = BackedValue::new(
                    PayloadProbe {
                        bytes,
                        cpu: cpu.clone(),
                        admitted,
                        observed: observed.clone(),
                        drops: drops.clone(),
                    },
                    [Some(hold), None],
                );
                DISPOSAL_TIME.store(5, Ordering::SeqCst);
                let error = failure(guarded.require_preparation());
                drop(guarded);
                error
            };
            assert_eq!(
                contract(&error),
                StorageContractError::WrongRetentionLifetime
            );
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            assert!(observed.load(Ordering::SeqCst) > baseline);
            let closing = cpu.snapshot().unwrap();
            assert_eq!(
                closing.usage().live(RenderResource::RetainedCpuBytes),
                baseline
            );
            assert_eq!(
                closing.scope_elapsed(RenderCpuScope::FrameBacking),
                if profile == (RenderProfile::Summary { timing: true }) {
                    Some(std::time::Duration::from_millis(7))
                } else {
                    None
                }
            );
        }
    }
}

#[test]
fn import_only_generic_owner_is_not_a_preparation_result() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let image = crate::Image::from_rgba(crate::Size::new(1.0, 1.0), vec![255, 0, 0, 255]).unwrap();
    let import = cpu
        .at(RenderCpuScope::ImagePayloads)
        .import_image(&image, None)
        .unwrap();
    let guarded = BackedValue::with_import(17_u8, [None, None], Some(import));
    assert_eq!(
        contract(&failure(guarded.require_preparation())),
        StorageContractError::WrongRetentionLifetime
    );
    drop(guarded);
}

#[test]
fn owning_iterator_keeps_credit_until_actual_remaining_values_are_dropped() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let baseline = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let mut values =
        AccountedVec::<u32>::new_in(&storage, 3, RenderResource::RetainedCpuBytes).unwrap();
    values.copy_slot(&[4, 5, 6]).unwrap().write();
    let bytes = (values.capacity() * 4) as u64;
    let mut iterator = values.into_iter();
    assert_eq!(iterator.next(), Some(4));
    assert_eq!(iterator.next_back(), Some(6));
    assert_eq!(iterator.len(), 1);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        baseline + bytes
    );
    drop(iterator);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        baseline
    );
}
