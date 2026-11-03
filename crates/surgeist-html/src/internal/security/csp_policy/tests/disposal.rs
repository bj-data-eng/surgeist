use super::*;
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
struct OriginalController {
    live: Cell<usize>,
    spent: Cell<u64>,
    refunds: Cell<usize>,
}
struct Custody<T> {
    value: Option<T>,
    original: Rc<OriginalController>,
    observation: Arc<Observation>,
    expected_drops_at_refund: usize,
}
impl<T> Custody<T> {
    fn acquire(
        value: T,
        layout: HtmlCspArtifactLayout,
        original: &Rc<OriginalController>,
        observation: &Arc<Observation>,
        expected: usize,
    ) -> Self {
        original.live.set(original.live.get() + 1);
        original
            .spent
            .set(original.spent.get() + layout.disposal().work_credits());
        Self {
            value: Some(value),
            original: original.clone(),
            observation: observation.clone(),
            expected_drops_at_refund: expected,
        }
    }
}
impl<T> Drop for Custody<T> {
    fn drop(&mut self) {
        drop(self.value.take());
        // This original-controller fixture refuses to refund before actual
        // native destruction when this occurrence was the last custodian.
        assert_eq!(
            self.observation.drops.load(Ordering::Relaxed),
            self.expected_drops_at_refund
        );
        self.original.live.set(self.original.live.get() - 1);
        self.original.refunds.set(self.original.refunds.get() + 1);
    }
}

#[test]
fn unpublished_abort_and_drop_preserve_association_and_destroy_only_abandoned_results() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let original_policy = association.policy().clone();
    let original_generation = association.generation().clone();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    let guard = owner
        .prepare_publication_with(&mut association, r, value, &mut allow)
        .unwrap();
    let (r, value) = guard.abort();
    assert_eq!(association.policy(), &original_policy);
    assert_eq!(association.generation(), &original_generation);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 0);
    let guard = owner
        .prepare_publication_with(&mut association, r, value, &mut allow)
        .unwrap();
    drop(guard);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 1);
    assert_eq!(association.policy(), &original_policy);
    assert_eq!(association.generation(), &original_generation);
    assert_eq!(p.observation.metadata.load(Ordering::Relaxed), 1);
}

#[test]
fn publish_has_no_callback_or_last_native_release_and_returns_every_owned_input() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    drop(publish(&mut owner, &mut association, r, value));
    let old_id = association.policy().id().clone();
    let r = request(&mut owner, &association, p.profile(), "2");
    let request_id = r.qualification().occurrence().clone();
    let value = prepared(&mut p, &r);
    let new_id = value.policy().id().clone();
    let mut callbacks = 0;
    let mut admission = |_| {
        callbacks += 1;
        Ok::<_, Stop>(())
    };
    let guard = owner
        .prepare_publication_with(&mut association, r, value, &mut admission)
        .unwrap();
    let before = callbacks;
    let before_metadata = p.observation.metadata.load(Ordering::Relaxed);
    let completion = guard.publish();
    assert_eq!(callbacks, before);
    assert_eq!(
        p.observation.metadata.load(Ordering::Relaxed),
        before_metadata
    );
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 0);
    let (receipt, retired, r, value) = completion.into_parts();
    assert_eq!(receipt.qualification().occurrence(), &request_id);
    assert_eq!(receipt.policy().id(), &new_id);
    assert_eq!(value.policy().id(), &new_id);
    assert_eq!(retired.policy().id(), &old_id);
    assert_eq!(r.prior_policy().id(), &old_id);
    drop(retired);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 0);
    drop(r);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 1);
    drop((receipt, value));
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 1);
    drop(association);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 2);
}

#[test]
fn final_snapshot_drops_native_before_original_receipt_refund_after_cancellation() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    drop(publish(&mut owner, &mut association, r, value));
    let source_control = Rc::new(OriginalController::default());
    let destination_control = Rc::new(OriginalController::default());
    let snapshot = association.clone();
    let snapshot_layout = snapshot.retained_layout().unwrap();
    let source = Custody::acquire(
        snapshot,
        snapshot_layout,
        &source_control,
        &p.observation,
        1,
    );
    let installed = owner
        .associate_document_with(
            &association,
            HtmlCspAssociationCause::Installation,
            &mut allow,
        )
        .unwrap();
    let installed_layout = installed.retained_layout().unwrap();
    let destination = Custody::acquire(
        installed,
        installed_layout,
        &destination_control,
        &p.observation,
        0,
    );
    drop(association);
    let spent = source_control.spent.get() + destination_control.spent.get();
    let mut stopped = Budget::unlimited();
    stopped.cancelled = true;
    assert_eq!(
        stopped.admit(HtmlCspLayout::read::<HtmlCspAssociation>(1)),
        Err(Stop::Cancelled)
    );
    drop(destination);
    assert_eq!(destination_control.live.get(), 0);
    assert_eq!(source_control.live.get(), 1);
    drop(source);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 1);
    assert_eq!(source_control.live.get(), 0);
    assert_eq!(source_control.refunds.get(), 1);
    assert_eq!(
        source_control.spent.get() + destination_control.spent.get(),
        spent
    );
    assert_eq!(stopped.first, Some(Stop::Cancelled));
    assert_eq!(stopped.spent, 0, "cleanup never polls or acquires new work");
}

#[test]
fn native_custody_cleanup_during_unwind_needs_no_fallible_callback() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    let observation = p.observation.clone();
    let original = Rc::new(OriginalController::default());
    // catch_unwind belongs only to this test's injected caller panic. Native
    // cleanup itself neither catches panic nor invokes a cleanup callback.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut admitted = None;
        let guard = owner
            .prepare_publication_with(&mut association, r, value, &mut |layout| {
                admitted = Some(layout);
                Ok::<_, Stop>(())
            })
            .unwrap();
        let _custody = Custody::acquire(
            guard,
            admitted.unwrap().retained(),
            &original,
            &observation,
            1,
        );
        panic!("injected caller unwind after complete admission");
    }));
    assert!(result.is_err());
    assert_eq!(observation.drops.load(Ordering::Relaxed), 1);
    assert_eq!(original.live.get(), 0);
    assert_eq!(original.refunds.get(), 1);
    assert!(original.spent.get() > 0);
    assert!(association.policy().profile().is_none());
}
