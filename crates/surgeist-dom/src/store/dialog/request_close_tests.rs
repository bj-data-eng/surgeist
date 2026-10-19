//! Canonical storage at its private owner; no fabricated native Window service.
use super::*;

fn fixture() -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            100_000,
            128,
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
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&dialog).unwrap().unwrap(),
    )
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4),
        &AtomicBool::new(false),
    )
    .unwrap()
    .0
}
fn publish(dom: &mut Dom, mut draft: Draft) {
    let prepared = draft.finish(dom, &AtomicBool::new(false)).unwrap();
    dom.publish(prepared, &AtomicBool::new(false)).unwrap();
}

#[test]
fn nullable_request_storage_preserves_utf16_snapshot_and_counts_each_occurrence() {
    let (mut dom, _, dialog) = fixture();
    let original = dom.snapshot();
    let value = DocumentString::from_utf16(vec![0xd800, 0, 13, 10, 0xdc00]);
    let mut staged = draft(&dom);
    let key = staged.resolve(dialog.into()).unwrap();
    let before_units = staged.units;
    staged
        .set_dialog_return_value(key, &value, dom.limits, &AtomicBool::new(false))
        .unwrap();
    staged
        .set_dialog_request_return_value(key, Some(&value), dom.limits, &AtomicBool::new(false))
        .unwrap();
    staged
        .set_dialog_request_flag(key, true, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        staged.units,
        before_units + 10,
        "two independent five-unit language slots"
    );
    let state = staged.dialog_state(key).unwrap();
    let family = crate::element::NativeFamily::Dialog(Arc::new(state.clone()));
    let retained = source::storage::family_payload(&family, || Ok(())).unwrap();
    assert_eq!(retained.units, value.shared_code_units().capacity() * 2);
    assert_eq!(
        retained.bytes,
        size_of::<DomDialogState>()
            + 2 * (size_of::<Vec<u16>>() + value.shared_code_units().capacity() * size_of::<u16>())
    );
    drop(family);
    publish(&mut dom, staged);
    let saved = dom.snapshot();
    let view = saved.node(dialog).unwrap().as_dialog().unwrap();
    assert_eq!(
        view.request_close_return_value().unwrap().code_units(),
        [0xd800, 0, 13, 10, 0xdc00]
    );
    assert_eq!(view.return_value(), &value);
    assert!(view.force_close_watcher_enabled());
    assert_eq!(
        original
            .node(dialog)
            .unwrap()
            .as_dialog()
            .unwrap()
            .request_close_return_value(),
        None
    );
    let mut staged = draft(&dom);
    let key = staged.resolve(dialog.into()).unwrap();
    staged
        .set_dialog_request_return_value(key, None, dom.limits, &AtomicBool::new(false))
        .unwrap();
    staged
        .set_dialog_request_source_null(key, &AtomicBool::new(false))
        .unwrap();
    staged
        .set_dialog_request_flag(key, false, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(staged.units, before_units + 5);
    publish(&mut dom, staged);
    let current = dom.snapshot();
    let view = current.node(dialog).unwrap().as_dialog().unwrap();
    assert_eq!(view.request_close_return_value(), None);
    assert!(!view.force_close_watcher_enabled());
    assert_eq!(view.return_value(), &value);
    assert_eq!(
        saved
            .node(dialog)
            .unwrap()
            .as_dialog()
            .unwrap()
            .request_close_return_value(),
        Some(&value)
    );
}

#[test]
fn adoption_preserves_request_state_and_clone_starts_with_fresh_null_false() {
    let (mut dom, original_document, dialog) = fixture();
    let mut staged = draft(&dom);
    let key = staged.resolve(dialog.into()).unwrap();
    staged
        .set_dialog_request_return_value(
            key,
            Some(&"retained".into()),
            dom.limits,
            &AtomicBool::new(false),
        )
        .unwrap();
    staged
        .set_dialog_request_flag(key, true, &AtomicBool::new(false))
        .unwrap();
    publish(&mut dom, staged);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let clone = batch.import_node(&before, dialog, &document, true);
    batch.adopt(dialog, &document);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let clone = commit.created(&clone).unwrap().unwrap();
    let current = dom.snapshot();
    assert_eq!(
        current.node(dialog).unwrap().owner_document(),
        Some(document)
    );
    let original = current.node(dialog).unwrap().as_dialog().unwrap();
    assert_eq!(
        original.request_close_return_value(),
        Some(&DocumentString::from("retained"))
    );
    assert!(original.force_close_watcher_enabled());
    let cloned = current.node(clone).unwrap().as_dialog().unwrap();
    assert_eq!(cloned.request_close_return_value(), None);
    assert_eq!(cloned.request_close_source(), None);
    assert!(!cloned.force_close_watcher_enabled());
    assert_eq!(
        before.node(dialog).unwrap().owner_document(),
        Some(original_document)
    );
}

#[test]
fn one_short_request_string_allowance_and_cancellation_leave_original_slots_unchanged() {
    for cancelled in [false, true] {
        let (dom, _, dialog) = fixture();
        let mut staged = draft(&dom);
        let key = staged.resolve(dialog.into()).unwrap();
        let units = staged.units;
        let mut limits = dom.limits;
        limits.units = units + 2;
        let cancel = AtomicBool::new(cancelled);
        let error = staged
            .set_dialog_request_return_value(key, Some(&"abc".into()), limits, &cancel)
            .unwrap_err();
        if cancelled {
            assert_eq!(error, DomError::Cancelled);
        } else {
            assert_eq!(
                error,
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: units + 2
                }
            );
        }
        assert!(
            staged
                .dialog_state(key)
                .unwrap()
                .request_return_value
                .is_none()
        );
        assert!(!staged.dialog_state(key).unwrap().force_watcher_enabled);
        assert_eq!(staged.units, units);
        assert_eq!(staged.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(staged.source_temporary_units.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn supplied_empty_request_is_distinct_from_null_and_can_be_cleared_without_changing_return_value() {
    let (mut dom, _, dialog) = fixture();
    let mut staged = draft(&dom);
    let key = staged.resolve(dialog.into()).unwrap();
    let units = staged.units;
    staged
        .set_dialog_request_return_value(
            key,
            Some(&DocumentString::default()),
            dom.limits,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(staged.units, units);
    publish(&mut dom, staged);
    let empty = dom.snapshot();
    assert_eq!(
        empty
            .node(dialog)
            .unwrap()
            .as_dialog()
            .unwrap()
            .request_close_return_value(),
        Some(&DocumentString::default())
    );
    let mut staged = draft(&dom);
    let key = staged.resolve(dialog.into()).unwrap();
    staged
        .set_dialog_request_return_value(key, None, dom.limits, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(staged.units, units);
    publish(&mut dom, staged);
    assert_eq!(
        dom.snapshot()
            .node(dialog)
            .unwrap()
            .as_dialog()
            .unwrap()
            .request_close_return_value(),
        None
    );
    assert_eq!(
        empty
            .node(dialog)
            .unwrap()
            .as_dialog()
            .unwrap()
            .request_close_return_value(),
        Some(&DocumentString::default())
    );
}

#[test]
fn force_flag_candidate_admits_both_live_request_backings_before_replacing_the_body() {
    let (dom, _, dialog) = fixture();
    for short in [true, false] {
        let cancel = AtomicBool::new(false);
        let mut staged = draft(&dom);
        let key = staged.resolve(dialog.into()).unwrap();
        let value = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        staged
            .set_dialog_request_return_value(key, Some(&value), dom.limits, &cancel)
            .unwrap();
        let expected = transaction::queues::admission_observation::dialog_fields(&staged, key);
        let boundary = transaction::queues::admission_observation::Binding::fields(
            key,
            expected,
            DomResource::SourceRetainedBytes,
            short,
        );
        let outcome = staged.set_dialog_request_flag(key, true, &cancel);
        let limit = boundary.limit();
        drop(boundary);
        if short {
            assert_eq!(
                outcome.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit
                }
            );
            assert!(!staged.dialog_state(key).unwrap().force_watcher_enabled);
        } else {
            outcome.unwrap();
            assert!(staged.dialog_state(key).unwrap().force_watcher_enabled);
        }
        assert_eq!(
            staged
                .dialog_state(key)
                .unwrap()
                .request_return_value
                .as_ref(),
            Some(&value)
        );
        assert_eq!(staged.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(staged.source_temporary_units.load(Ordering::Relaxed), 0);
    }
}
