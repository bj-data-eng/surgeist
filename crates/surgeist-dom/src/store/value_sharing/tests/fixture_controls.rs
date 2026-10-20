use super::super::fixture::{Layers, control};
use super::*;
use crate::{
    Dom, DomBatch, DomLimits, DomResource, DomSourceEdit, DomSourceLimits, DomSourceProgress,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

const MODES: [Layers; 3] = [Layers::CallerOnly, Layers::BanksOnly, Layers::Combined];
const RAW: &[u16] = &[107, 0xd800, 0, 0xdc00];

fn raw(capacity: usize) -> DocumentString {
    let mut units = Vec::with_capacity(capacity);
    units.extend_from_slice(RAW);
    DocumentString::from_utf16(units)
}

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            64,
            1_000_000,
            128,
            0,
            0,
            64,
            8_000_000,
            DomNameLimits::new(32_000_000, 16_384, 10_000_000, 4_000_000).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4)
}

#[test]
fn disabled_layers_do_not_read_hash_or_activate_the_optional_owner() {
    for mode in [Layers::CallerOnly, Layers::BanksOnly] {
        let f = fixture();
        let guard = control(&f.pool, mode);
        let session = admitted(|a| f.pool.attach(a)).unwrap();
        let baseline = f.ledger.total();
        let text = raw(17);
        let mut no_actions = Admission {
            global: &mut |_| panic!("a disabled layer cannot retain a candidate"),
            step: &mut || panic!("a disabled layer cannot inspect input or acquire a root"),
        };
        assert!(
            session
                .share_string(&text, &mut no_actions)
                .unwrap()
                .is_none()
        );
        session
            .seed_string(f.name.local_name(), &mut no_actions)
            .unwrap();
        if mode == Layers::CallerOnly {
            let bank = value(&f, RAW, 7);
            assert!(matches!(
                session.probe_bank(bank.entries(), &mut no_actions).unwrap(),
                BankProbe::Bypassed
            ));
            session.offer_bank(&bank, None, &mut no_actions).unwrap();
        }
        assert!(session.is_dormant());
        assert!(lock(&f.pool.0).roots.is_none());
        assert_eq!(f.ledger.total(), baseline);
        drop(session);
        drop(guard);
    }
}

#[test]
fn owner_qualified_control_leaves_other_owners_and_threads_combined() {
    let f = fixture();
    let other = fixture();
    let bank = value(&f, RAW, 7);
    let guard = control(&f.pool, Layers::CallerOnly);

    let other_session = admitted(|a| other.pool.attach(a)).unwrap();
    let other_bank = value(&other, RAW, 9);
    offer(&other_session, &other_bank);
    assert!(
        hit(&other_session, &value(&other, RAW, 1))
            .storage()
            .same_storage(&other_bank)
    );
    seal(other_session);

    // The controlled owner itself uses its ordinary combined behavior on a
    // different thread. Join is the boundary; there is no timing assumption.
    std::thread::scope(|scope| {
        let pool = &f.pool;
        let bank = &bank;
        scope
            .spawn(move || {
                let session = admitted(|a| pool.attach(a)).unwrap();
                offer(&session, bank);
                assert!(hit(&session, bank).storage().same_storage(bank));
                seal(session);
            })
            .join()
            .unwrap();
    });
    let session = admitted(|a| f.pool.attach(a)).unwrap();
    assert!(matches!(
        admitted(|a| session.probe_bank(bank.entries(), a)).unwrap(),
        BankProbe::Bypassed
    ));
    assert!(session.is_dormant());
    drop(guard);
    assert!(
        hit(&session, &value(&f, RAW, 1))
            .storage()
            .same_storage(&bank)
    );
    seal(session);
}

#[test]
fn panic_releases_the_control_and_nested_installation_preserves_its_outer_scope() {
    let f = fixture();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = control(&f.pool, Layers::CallerOnly);
        let session = admitted(|a| f.pool.attach(a)).unwrap();
        let nested = catch_unwind(AssertUnwindSafe(|| control(&f.pool, Layers::Combined)));
        assert!(nested.is_err());
        assert!(matches!(
            admitted(|a| session.probe_bank(value(&f, RAW, 1).entries(), a)).unwrap(),
            BankProbe::Bypassed
        ));
        std::panic::panic_any(73u8);
    }));
    assert_eq!(result.unwrap_err().downcast_ref::<u8>(), Some(&73));

    let guard = control(&f.pool, Layers::Combined);
    let session = admitted(|a| f.pool.attach(a)).unwrap();
    let bank = value(&f, RAW, 5);
    offer(&session, &bank);
    assert!(
        hit(&session, &value(&f, RAW, 1))
            .storage()
            .same_storage(&bank)
    );
    seal(session);
    drop(guard);
}

#[test]
fn each_mode_preserves_paired_checkpoint_restore_and_closing() {
    for mode in MODES {
        let f = fixture();
        let guard = control(&f.pool, mode);
        let session = admitted(|a| f.pool.attach(a)).unwrap();
        let base = value(&f, &[11], 3);
        let later = value(&f, &[22], 5);
        let base_text = DocumentString::from("base-text");
        let later_text = DocumentString::from("later-text");
        offer(&session, &base);
        assert!(
            admitted(|a| session.share_string(&base_text, a))
                .unwrap()
                .is_none()
        );
        let mark = admitted(|a| session.checkpoint(a)).unwrap();
        offer(&session, &later);
        assert!(
            admitted(|a| session.share_string(&later_text, a))
                .unwrap()
                .is_none()
        );
        mark.restore();
        if mode == Layers::CallerOnly {
            assert!(matches!(
                admitted(|a| session.probe_bank(base.entries(), a)).unwrap(),
                BankProbe::Bypassed
            ));
            assert!(session.is_dormant());
        } else {
            assert!(hit(&session, &base).storage().same_storage(&base));
            assert!(matches!(
                admitted(|a| session.probe_bank(later.entries(), a)).unwrap(),
                BankProbe::Miss(_)
            ));
        }
        let selected =
            admitted(|a| session.share_string(&DocumentString::from("base-text"), a)).unwrap();
        if mode == Layers::Combined {
            assert!(Arc::ptr_eq(
                selected.as_ref().unwrap().value().shared_code_units(),
                base_text.shared_code_units()
            ));
        } else {
            assert!(selected.is_none());
        }
        assert!(
            admitted(|a| session.share_string(&DocumentString::from("later-text"), a))
                .unwrap()
                .is_none()
        );
        drop(selected);
        seal(session);
        assert_raw(&f);
        drop(guard);
    }
}

#[test]
fn native_values_keep_mandatory_atoms_and_select_only_the_enabled_backings() {
    let cancel = AtomicBool::new(false);
    for mode in MODES {
        let mut dom = dom();
        let guard = control(&dom.value_sharing, mode);
        let caller_shared = DocumentString::from("caller-backed");
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let first = batch.create_element(&document, "cell");
        batch.set_attribute(&first, "data-x", raw(17));
        batch.set_attribute(&first, "id", "one");
        let equal = batch.create_element(&document, "cell");
        batch.set_attribute(&equal, "data-x", raw(4));
        batch.set_attribute(&equal, "id", "one");
        let different = batch.create_element(&document, "cell");
        batch.set_attribute(&different, "data-x", raw(4));
        batch.set_attribute(&different, "id", "two");
        let text = batch.create_text(&document, raw(23));
        let equal_text = batch.create_text(&document, raw(4));
        let caller_first = batch.create_text(&document, caller_shared.clone());
        let caller_second = batch.create_text(&document, caller_shared.clone());
        let done = dom.apply(batch, &cancel).unwrap();
        let first = done.created(&first).unwrap().unwrap();
        let equal = done.created(&equal).unwrap().unwrap();
        let different = done.created(&different).unwrap().unwrap();
        let text = done.created(&text).unwrap().unwrap();
        let equal_text = done.created(&equal_text).unwrap().unwrap();
        let caller_first = done.created(&caller_first).unwrap().unwrap();
        let caller_second = done.created(&caller_second).unwrap().unwrap();
        let before = dom.snapshot();
        let left = before.node(first).unwrap().as_element().unwrap();
        let same = before.node(equal).unwrap().as_element().unwrap();
        let other = before.node(different).unwrap().as_element().unwrap();
        assert_ne!(first, equal);
        assert_eq!(left.name().local_name_atom(), same.name().local_name_atom());
        assert_eq!(
            left.attributes()[0].name().local_name_atom(),
            other.attributes()[0].name().local_name_atom()
        );
        assert_eq!(
            left.attributes()[0].name().local_name().code_units(),
            [100, 97, 116, 97, 45, 120]
        );
        assert!(left.attributes()[0].name().namespace_uri().is_none());
        assert!(left.attributes()[0].name().prefix().is_none());
        assert_eq!(left.attributes()[0].value().code_units(), RAW);
        assert_eq!(same.attributes()[1].value().code_units(), [111, 110, 101]);
        assert_eq!(other.attributes()[1].value().code_units(), [116, 119, 111]);
        assert_eq!(
            left.same_attribute_storage(same),
            mode != Layers::CallerOnly
        );
        assert!(!left.same_attribute_storage(other));
        assert_eq!(
            Arc::ptr_eq(
                left.attributes()[0].value().shared_code_units(),
                other.attributes()[0].value().shared_code_units()
            ),
            mode == Layers::Combined
        );
        assert_eq!(
            Arc::ptr_eq(
                before
                    .node(text)
                    .unwrap()
                    .data()
                    .unwrap()
                    .shared_code_units(),
                before
                    .node(equal_text)
                    .unwrap()
                    .data()
                    .unwrap()
                    .shared_code_units()
            ),
            mode == Layers::Combined
        );
        for id in [caller_first, caller_second] {
            assert!(Arc::ptr_eq(
                before.node(id).unwrap().data().unwrap().shared_code_units(),
                caller_shared.shared_code_units()
            ));
        }

        let usage = dom.usage();
        let mut noop = DomBatch::new();
        noop.set_attribute(first, "data-x", raw(4));
        noop.replace_data(text, 0, usize::MAX, raw(4));
        let done = dom.apply(noop, &cancel).unwrap();
        let after = dom.snapshot();
        assert!(done.changes().changes().is_empty());
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(left.same_attribute_storage(after.node(first).unwrap().as_element().unwrap()));
        assert!(Arc::ptr_eq(
            before
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units(),
            after
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units()
        ));

        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_data(text, 0, usize::MAX, "changed"),
                source_limits(),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert_eq!(
            dom.snapshot()
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .code_units(),
            [99, 104, 97, 110, 103, 101, 100]
        );
        assert_eq!(before.node(text).unwrap().data().unwrap().code_units(), RAW);
        assert_eq!(
            dom.snapshot()
                .node(equal_text)
                .unwrap()
                .data()
                .unwrap()
                .code_units(),
            RAW
        );
        // End the diagnostic Weak before any retained snapshot/owner teardown.
        drop(guard);
    }
}

#[test]
fn every_mode_preserves_mandatory_source_refusal_and_unpublished_cancellation() {
    let cancel = AtomicBool::new(false);
    for mode in MODES {
        let mut dom = dom();
        let guard = control(&dom.value_sharing, mode);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let text = batch.create_text(&document, "old");
        let done = dom.apply(batch, &cancel).unwrap();
        let text = done.created(&text).unwrap().unwrap();
        let before = dom.snapshot();
        let refused = match dom.begin_source_edit(
            DomSourceEdit::replace_data(text, 0, usize::MAX, "new"),
            DomSourceLimits::new(128_000_000, 0, 20_000_000, 64, 4096, 4096, 4),
            &cancel,
        ) {
            Err(error) => error,
            Ok(_) => panic!("nonempty mandatory Source input cannot fit zero retained units"),
        };
        assert_eq!(
            refused,
            DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                limit: 0
            }
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        let mut batch = DomBatch::new();
        batch.replace_data(text, 0, usize::MAX, "unpublished");
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(Arc::ptr_eq(
            dom.snapshot()
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units(),
            before
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units()
        ));

        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_data(text, 0, usize::MAX, "new"),
                source_limits(),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert_eq!(
            dom.snapshot()
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .code_units(),
            [110, 101, 119]
        );
        assert_eq!(
            before.node(text).unwrap().data().unwrap().code_units(),
            [111, 108, 100]
        );
        drop(guard);
    }
}
