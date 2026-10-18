//! New private fixed-atom custody characterization with literal lifetimes.
use super::*;

#[test]
fn formed_utf16_factories_report_completed_prefix_before_original_work_refusal() {
    use scale_metrics::{Hooks, Measurement};
    if !scale_metrics::ENABLED {
        return;
    }
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(DomLimits::new(8, 10_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    draft.source_limits = Some(crate::DomSourceLimits::new(
        16_000_000, 1_000_000, 1_000_000, 32, 128, 128, 1,
    ));
    let baseline = draft.custody_ledger().total();
    for utf8 in [false, true] {
        let mut saw_zero = false;
        let mut saw_prefix = false;
        let mut completed = false;
        for allowance in 0..64 {
            let mut budget = Formation::new(&draft, &cancel).unwrap().unwrap();
            let limit = budget.work.load(Ordering::Relaxed) + allowance;
            budget.work_limit = limit;
            let measurement = Measurement::begin().unwrap();
            let result = if utf8 {
                string("AZ\0b", Some(&budget), &cancel)
            } else {
                units(&[0x0041, 0xd800, 0x005a, 0], true, Some(&budget), &cancel)
            };
            let counters = measurement.finish(Hooks::default()).unwrap();
            match result {
                Ok(value) => {
                    let expected: &[u16] = if utf8 {
                        &[0x0041, 0x005a, 0, 0x0062]
                    } else {
                        &[0x0061, 0xd800, 0x007a, 0]
                    };
                    assert_eq!(value.shared_code_units().as_slice(), expected);
                    assert_eq!(counters.sequence_materialized_bytes, 4 * size_of::<u16>());
                    assert_eq!(counters.sequence_materializations, 1);
                    completed = true;
                    drop(value);
                }
                Err(error) => {
                    assert_eq!(error, DomError::SourceWorkLimit { limit });
                    assert_eq!(budget.work.load(Ordering::Relaxed), limit);
                    let bytes = counters.sequence_materialized_bytes;
                    assert!(bytes < 4 * size_of::<u16>());
                    saw_zero |= bytes == 0;
                    saw_prefix |= bytes > 0;
                    assert_eq!(counters.sequence_materializations, usize::from(bytes != 0));
                }
            }
            drop(budget);
            assert_eq!(draft.custody_ledger().total(), baseline);
            if completed {
                break;
            }
        }
        assert!(completed && saw_zero && saw_prefix);
    }
}

#[test]
fn original_counter_alias_and_later_readoption_preserve_identity_and_spent_work() {
    let ledger = custody::Ledger::default();
    let _administration = ledger
        .retain(custody::Ledger::administration_usage(), |_| Ok(()))
        .unwrap();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let (work, original) = SourceWorkCustody::allocate(3, &registry, |_| Ok(())).unwrap();
    let identity = original.identity();
    let retained = ledger.total();
    let alias = SourceWorkCustody::adopt_existing(identity, &work, &registry, |_| Ok(())).unwrap();
    assert!(Arc::ptr_eq(original.work(), alias.work()));
    assert_eq!(alias.identity(), identity);
    assert_eq!(ledger.total(), retained);
    work.fetch_add(4, Ordering::Relaxed);
    assert_eq!(alias.work().load(Ordering::Relaxed), 7);
    drop(original);
    assert_eq!(ledger.total(), retained);
    drop(alias);
    registry.reconcile_releases();
    let uncontrolled = ledger.total();
    assert!(uncontrolled.bytes() < retained.bytes());
    // The caller's raw historical counter retains no controlled debit. A later
    // true owner adopts its original issuer ID, rather than minting another ID
    // or allocating a replacement counter initialized from a loaded value.
    let adopted =
        SourceWorkCustody::adopt_existing(identity, &work, &registry, |_| Ok(())).unwrap();
    assert_eq!(adopted.identity(), identity);
    assert!(Arc::ptr_eq(adopted.work(), &work));
    assert_eq!(adopted.work().load(Ordering::Relaxed), 7);
    assert!(ledger.total().bytes() > uncontrolled.bytes());
}

#[test]
fn carried_output_keeps_original_counter_and_mirrors_controlled_after_source_drops() {
    let ledger = custody::Ledger::default();
    let _administration = ledger
        .retain(custody::Ledger::administration_usage(), |_| Ok(()))
        .unwrap();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let (work, original) = SourceWorkCustody::allocate(5, &registry, |_| Ok(())).unwrap();
    let mirrors = SourceMirrors::allocate(&registry, |_| Ok(())).unwrap();
    let bytes = mirrors.bytes();
    let units = mirrors.units();
    let debit = ledger
        .retain(custody::Usage::new(16, 0), |_| Ok(()))
        .unwrap();
    let mut output = Vec::<u8>::with_capacity(16);
    output.extend_from_slice(b"carried output");
    assert_eq!(output.capacity(), 16);
    bytes.store(16, Ordering::Relaxed);
    let charge = super::super::checkpoint::Charge::payload(
        SourceStorage::new(ledger.clone()),
        bytes.clone(),
        16,
        units.clone(),
        0,
        Some(debit),
        WorkDependencies {
            _source: original.clone(),
            _event: None,
            _mirrors: Some(mirrors.clone()),
        },
    );
    let retained = ledger.total();
    drop(original);
    drop(mirrors);
    assert_eq!(ledger.total(), retained);
    assert_eq!(work.fetch_add(2, Ordering::Relaxed), 5);
    assert_eq!(work.load(Ordering::Relaxed), 7);
    assert_eq!(bytes.load(Ordering::Relaxed), 16);
    assert_eq!(output.as_slice(), b"carried output");
    drop(output);
    drop(charge);
    registry.reconcile_releases();
    assert_eq!(bytes.load(Ordering::Relaxed), 0);
    assert_eq!(units.load(Ordering::Relaxed), 0);
    assert!(ledger.total().bytes() < retained.bytes());
    // Caller aliases are numerical values only, after the final owning output
    // releases its actual fixed targets and payload custody.
    assert_eq!(work.load(Ordering::Relaxed), 7);
}

#[test]
fn selected_payload_transfer_ends_mirrors_without_releasing_its_occurrence() {
    let ledger = custody::Ledger::default();
    let _administration = ledger
        .retain(custody::Ledger::administration_usage(), |_| Ok(()))
        .unwrap();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let (_, work) = SourceWorkCustody::allocate(0, &registry, |_| Ok(())).unwrap();
    let mirrors = SourceMirrors::allocate(&registry, |_| Ok(())).unwrap();
    let fixed = ledger.total();
    let debit = ledger
        .retain(custody::Usage::new(24, 12), |_| Ok(()))
        .unwrap();
    let value = Vec::<u16>::with_capacity(12);
    assert_eq!(value.capacity(), 12);
    mirrors.bytes().store(24, Ordering::Relaxed);
    mirrors.units().store(12, Ordering::Relaxed);
    let charge = super::super::checkpoint::Charge::payload(
        SourceStorage::new(ledger.clone()),
        mirrors.bytes(),
        24,
        mirrors.units(),
        12,
        Some(debit),
        WorkDependencies {
            _source: work.clone(),
            _event: None,
            _mirrors: Some(mirrors.clone()),
        },
    );
    let before = ledger.total();
    let owner = custody::Held::new(value, charge.into_debit().unwrap());
    assert_eq!(mirrors.bytes().load(Ordering::Relaxed), 0);
    assert_eq!(mirrors.units().load(Ordering::Relaxed), 0);
    // Keep the original work target controlled independently of this field,
    // as the receiving Draft does during a selected-field handoff.
    assert_eq!(ledger.total(), before);
    assert_eq!(before.bytes(), fixed.bytes() + 24);
    assert_eq!(owner.value().capacity(), 12);
    drop(owner);
    assert_eq!(ledger.total(), fixed);
}

#[test]
fn pi_factory_same_owner_cursor_peak_is_exact_and_one_short_preserves_inputs() {
    use crate::DomSourceLimits;
    use crate::node_values::{DomProcessingInstruction, source_observation};
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(DomLimits::new(16, 100_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    let target = DocumentString::from("pi");
    let data = DocumentString::from(format!("a='{}' b='literal'", "x".repeat(4096)));
    let original = data.shared_code_units().clone();
    let mut peak = None;
    for short in [None, Some(false), Some(true)] {
        let byte_limit = short.map_or(16_000_000, |short| peak.unwrap() - usize::from(short));
        draft.source_limits = Some(DomSourceLimits::new(
            byte_limit, 1_000_000, 10_000_000, 64, 4096, 4096, 1,
        ));
        let baseline = draft.custody_ledger().total();
        let source = Formation::new(&draft, &cancel).unwrap().unwrap();
        source_observation::start();
        let ledger = draft.custody_ledger().clone();
        let _binding = source_observation::bind(
            move || {
                let live = ledger.total();
                (live.bytes(), live.units())
            },
            byte_limit,
            1_000_000,
        );
        let result = DomProcessingInstruction::new(&target, &data, Some(&source), &cancel);
        let actual = source_observation::finish();
        if short == Some(true) {
            assert!(matches!(result, Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes, limit,
            }) if limit == byte_limit));
            if let Some(actual) = actual {
                assert!(actual.required <= actual.limit);
            }
        } else {
            let value = result.unwrap();
            let attributes: Vec<_> = value.attributes().collect();
            assert_eq!(attributes.len(), 2);
            assert_eq!(attributes[0].0, &DocumentString::from("a"));
            assert_eq!(attributes[0].1, &DocumentString::from("x".repeat(4096)));
            assert_eq!(
                attributes[1],
                (&DocumentString::from("b"), &DocumentString::from("literal"))
            );
            let actual = actual.expect("actual cursor backing growth");
            if short == Some(false) {
                assert_eq!(actual.required, byte_limit);
            } else {
                peak = Some(actual.required);
            }
            drop(value);
        }
        drop(source);
        assert_eq!(draft.custody_ledger().total(), baseline);
        assert!(Arc::ptr_eq(&original, data.shared_code_units()));
    }
}

#[test]
fn actual_formation_and_unformed_reserves_report_allocated_capacity_and_moved_members() {
    if !scale_metrics::ENABLED {
        return;
    }
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(DomLimits::new(8, 10_000, 32, 0, 0, 8, 10_000).unwrap()).unwrap();
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    draft.source_limits = Some(crate::DomSourceLimits::new(
        16_000_000, 1_000_000, 1_000_000, 32, 128, 128, 1,
    ));
    let budget = Formation::new(&draft, &cancel).unwrap().unwrap();
    let measurement = scale_metrics::Measurement::begin().unwrap();
    let mut values = Vec::<u64>::new();
    reserve(&mut values, 4, Some(&budget), &cancel).unwrap();
    let first = values.capacity();
    values.extend_from_slice(&[3, 5]);
    reserve(&mut values, first, Some(&budget), &cancel).unwrap();
    let second = values.capacity();
    assert_eq!(values, [3, 5]);
    let mut ordinary = Vec::<u32>::new();
    reserve(&mut ordinary, 5, None, &cancel).unwrap();
    let raw = utf16_buffer(7, 32, None, &cancel).unwrap();
    let expected = (first + second) * size_of::<u64>()
        + ordinary.capacity() * size_of::<u32>()
        + raw.capacity() * size_of::<u16>();
    let counters = measurement.finish(scale_metrics::Hooks::default()).unwrap();
    assert_eq!(counters.structural_allocations, 4);
    assert_eq!(counters.structural_bytes, expected);
    assert_eq!(counters.sequence_materializations, 1);
    assert_eq!(counters.sequence_materialized_bytes, 2 * size_of::<u64>());
    drop(values);
    drop(budget);
}

#[test]
fn actual_family_targets_count_replacements_and_cow_but_not_unique_mutation() {
    use scale_metrics::{Hooks, Measurement};
    if !scale_metrics::ENABLED {
        return;
    }
    let native = |name: &str| {
        crate::DomElement::native_components(
            &crate::QualifiedName::new(crate::Namespace::html(), None, name),
            crate::DomDocumentEnvironment::new(crate::DomNativeInputPlatform::headless()),
            &AtomicBool::new(false),
            None,
        )
        .unwrap()
    };
    let mut input = native("input");
    let original = input.clone();
    let measurement = Measurement::begin().unwrap();
    input.make_family_unique();
    let counters = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counters.structural_allocations, 1);
    assert_eq!(counters.structural_bytes, size_of::<crate::DomInputState>());
    assert_eq!(input.input_state(), original.input_state());
    let measurement = Measurement::begin().unwrap();
    input.make_family_unique();
    input.input_state_mut().unwrap();
    let counters = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counters.structural_allocations, 0);

    let mut dialog = native("dialog");
    let dialog_state = dialog.dialog_state().unwrap().clone();
    let mut style = native("style");
    let style_state = style.style_state().unwrap().clone();
    let measurement = Measurement::begin().unwrap();
    dialog.set_dialog_state(dialog_state);
    style.set_style_state(style_state);
    let counters = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counters.structural_allocations, 2);
    assert_eq!(
        counters.structural_bytes,
        size_of::<crate::dialog::DomDialogState>() + size_of::<crate::DomStyleState>()
    );
}
