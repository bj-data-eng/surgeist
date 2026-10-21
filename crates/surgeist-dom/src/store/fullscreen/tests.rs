//! Owning unavailable-history, raw/pending and original checkpoint boundaries.
//! Private structural inputs do not certify popover or platform execution.
use super::*;
use crate::dialog::DocumentInteractiveState;
use crate::{DomSourceEdit, DomSourceLimits, DomTopLayerUaRule, Namespace};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default)]
struct Reads {
    calls: usize,
    work: u64,
}
thread_local! { static READS: Cell<Option<Reads>> = const { Cell::new(None) }; }
pub(super) struct DocumentRead {
    work: Arc<AtomicU64>,
    start: u64,
}
impl DocumentRead {
    pub(super) fn begin(draft: &Draft) -> Self {
        Self {
            work: Arc::clone(draft.source_work.source_arc()),
            start: draft.source_work.load(Ordering::Relaxed),
        }
    }
}
impl Drop for DocumentRead {
    fn drop(&mut self) {
        if let Some(mut reads) = READS.get() {
            reads.calls += 1;
            reads.work += self.work.load(Ordering::Relaxed) - self.start;
            READS.set(Some(reads));
        }
    }
}
struct Observation;
impl Observation {
    fn begin() -> Self {
        assert!(READS.get().is_none());
        READS.set(Some(Reads::default()));
        Self
    }
    fn finish(self) -> Reads {
        READS.take().unwrap()
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        READS.set(None);
    }
}

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn policy(outputs: usize, phases: u64) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, phases, outputs, 0, 1)
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            100_000,
            128,
            128,
            128,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let first = batch.create_element_ns(&document, Namespace::none(), "first");
    let other = batch.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&first).unwrap().unwrap(),
        commit.created(&other).unwrap().unwrap(),
    )
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        policy(128, 16),
        &cancel(),
    )
    .unwrap()
    .0
}
fn publish(dom: &mut Dom, draft: &mut Draft) {
    let prepared = draft.finish(dom, &cancel()).unwrap();
    dom.publish(prepared, &cancel()).unwrap();
}

#[test]
#[ignore = "requires an instrumented DOM build; measures the real Source no-op"]
fn actual_source_unfullscreen_with_no_flagged_members_does_not_scan_unrelated_raw_members() {
    scale_metrics::require_enabled().expect("compile with SURGEIST_DOM_STORAGE_COUNTERS=1");
    for unrelated in [0, 256] {
        let cancel = cancel();
        let mut dom = Dom::new(
            DomLimits::new(
                1024,
                1_000_000,
                2048,
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
        let tickets: Vec<_> = (0..unrelated)
            .map(|_| batch.create_element_ns(&document, Namespace::html(), "div"))
            .collect();
        let setup = dom.apply(batch, &cancel).unwrap();
        let document = setup.document(&document).unwrap();
        let members: Vec<_> = tickets
            .iter()
            .map(|ticket| setup.created(ticket).unwrap().unwrap())
            .collect();
        let mut staged = draft(&dom);
        for member in &members {
            let key = staged.resolve((*member).into()).unwrap();
            staged.add_top_layer(key, &cancel).unwrap();
        }
        publish(&mut dom, &mut staged);
        let old = dom.snapshot();
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::unfullscreen_document_model(document),
                policy(4096, 16),
                &cancel,
            )
            .unwrap();
        let observation = Observation::begin();
        let crate::DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("actual zero-participant model command completes without a service")
        };
        assert!(done.commit().changes().changes().is_empty());
        let reads = observation.finish();
        assert!(
            reads.calls > 0,
            "real Source must execute the document model query"
        );
        let work = reads.work;
        assert_eq!(lease.snapshot().stamp(), old.stamp());
        assert_eq!(
            lease
                .snapshot()
                .document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            members
        );
        assert!(
            lease
                .snapshot()
                .document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .pending_removals()
                .is_empty()
        );
        for member in &members {
            assert!(
                !lease
                    .snapshot()
                    .node(*member)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .fullscreen_flag()
            );
            assert!(
                !old.node(*member)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .fullscreen_flag()
            );
        }
        assert!(
            work <= 64,
            "zero flagged participants and unchanged raw sequence, unrelated={unrelated}, actual Source work={work}"
        );
    }
}

#[test]
fn unavailable_document_lists_refuse_the_first_read_without_flag_or_layer_publication() {
    let (mut dom, document, first, _) = fixture();
    let mut staged = draft(&dom);
    let document_key = staged.resolve(document.into()).unwrap();
    // Actual typed unknown-history input; no showing popover is invented.
    {
        let mut state = staged
            .interactive_document_mut(document_key, &cancel())
            .unwrap();
        *state = Some(DocumentInteractiveState::Unavailable);
        state.commit(&cancel()).unwrap();
    }
    publish(&mut dom, &mut staged);
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::fullscreen_element_model(first),
            policy(128, 1),
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        lease.advance(&cancel()).unwrap_err(),
        DomError::FullscreenModelUnavailable {
            step: Step::ReadTopLayer
        }
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(
        !lease
            .snapshot()
            .node(first)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .fullscreen_model_element()
            .unwrap_err(),
        DomError::DialogDocumentStateUnavailable
    );
}

#[test]
fn pending_flagged_member_counts_for_simple_document_while_unflagged_raw_members_survive_unfullscreen()
 {
    let (mut dom, document, first, other) = fixture();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(first);
    dom.apply(batch, &cancel()).unwrap();
    let mut staged = draft(&dom);
    let first_key = staged.resolve(first.into()).unwrap();
    let other_key = staged.resolve(other.into()).unwrap();
    // Exercise canonical list primitives, without claiming ShowModal/Close ran.
    staged.add_top_layer(other_key, &cancel()).unwrap();
    staged
        .request_top_layer_removal(first_key, &cancel())
        .unwrap();
    publish(&mut dom, &mut staged);
    let before = dom.snapshot();
    let view = before.document(document).unwrap();
    assert_eq!(view.fullscreen_model_element().unwrap(), Some(first));
    assert!(view.is_simple_fullscreen_document().unwrap());
    assert_eq!(
        view.top_layer().unwrap().ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(other)
        ]
    );
    let mut batch = DomBatch::new();
    batch.unfullscreen_document_model(document);
    dom.apply(batch, &cancel()).unwrap();
    let after = dom.snapshot();
    let view = after.document(document).unwrap();
    assert_eq!(view.top_layer().unwrap().raw_members(), [other]);
    assert!(view.top_layer().unwrap().pending_removals().is_empty());
    assert!(!view.fullscreen().unwrap());
    assert!(!view.is_simple_fullscreen_document().unwrap());
    assert!(
        before
            .node(first)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
}

#[test]
fn checkpoint_restores_model_flags_and_lists_after_the_real_unfullscreen_command() {
    let (mut dom, document, first, _) = fixture();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(first);
    dom.apply(batch, &cancel()).unwrap();
    let mut staged = draft(&dom);
    let first_key = staged.resolve(first.into()).unwrap();
    let document_key = staged.resolve(document.into()).unwrap();
    let checkpoint = staged.source_checkpoint(0, 0, &cancel()).unwrap();
    staged
        .unfullscreen_element_model(first_key, &cancel())
        .unwrap();
    assert!(!staged.element(first_key).unwrap().fullscreen_flag());
    assert!(
        staged
            .fullscreen_raw_members(document_key, &cancel())
            .unwrap()
            .is_empty()
    );
    checkpoint.restore(&mut staged);
    assert!(staged.element(first_key).unwrap().fullscreen_flag());
    assert_eq!(
        staged
            .fullscreen_raw_members(document_key, &cancel())
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![first_key]
    );
    assert_eq!(staged.select_temporary_bytes.load(Ordering::Relaxed), 0);
    publish(&mut dom, &mut staged);
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .fullscreen_model_element()
            .unwrap(),
        Some(first)
    );
}

#[test]
fn detached_adoption_preflight_does_not_scan_unrelated_raw_or_pending_members() {
    let cancel = cancel();
    let generous = DomSourceLimits::new(1_000_000_000, 16_000_000, 100_000_000, 16, 4096, 0, 1);
    let form = |dom: &Dom| {
        let batch = DomBatch::new();
        Draft::new_source(dom, batch.owner.clone(), &batch.commands, generous, &cancel)
            .unwrap()
            .0
    };
    for unrelated in [0, 4096] {
        let mut dom = Dom::new(
            DomLimits::new(
                8192,
                1_000_000,
                16_384,
                0,
                0,
                16,
                100_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let origin = batch.create_document();
        let destination = batch.create_document();
        let subject = batch.create_element_ns(&origin, Namespace::html(), "div");
        let tickets: Vec<_> = (0..unrelated)
            .map(|_| batch.create_element_ns(&origin, Namespace::html(), "div"))
            .collect();
        let setup = dom.apply(batch, &cancel).unwrap();
        let origin = setup.document(&origin).unwrap();
        let destination = setup.document(&destination).unwrap();
        let subject = setup.created(&subject).unwrap().unwrap();
        let members: Vec<_> = tickets
            .iter()
            .map(|ticket| setup.created(ticket).unwrap().unwrap())
            .collect();
        let mut staged = form(&dom);
        let owner = staged.resolve(origin.into()).unwrap();
        let keys: Vec<_> = members
            .iter()
            .map(|member| staged.resolve((*member).into()).unwrap())
            .collect();
        // Construct actual canonical relations through their admitted owner.
        // These native Elements are not claimed to have completed modal Show.
        for role in [
            transaction::RelationField::TopLayer,
            transaction::RelationField::PendingTopLayerRemovals,
        ] {
            staged
                .relations_insert(owner, role, 0, &keys, &cancel)
                .unwrap();
        }
        publish(&mut dom, &mut staged);
        let old = dom.snapshot();
        let mut staged = form(&dom);
        let subject_key = staged.resolve(subject.into()).unwrap();
        let origin_key = staged.resolve(origin.into()).unwrap();
        let destination_key = staged.resolve(destination.into()).unwrap();
        assert_eq!(staged.node(subject_key).unwrap().document(), origin_key);
        assert_eq!(staged.node(subject_key).unwrap().parent(), None);
        assert!(staged.node(subject_key).unwrap().children().is_empty());
        assert!(staged.node(subject_key).unwrap().shadow_root().is_none());
        assert!(!staged.element(subject_key).unwrap().fullscreen_flag());
        let mut results = Vec::new();
        for early in [true, false] {
            // Cap the same already-formed Source Draft immediately before the
            // real preflight, excluding formation and any actual removal walk.
            let spent = staged.source_work.load(Ordering::Relaxed);
            assert_eq!(
                staged.preflight_top_layer_adoption(
                    subject_key,
                    destination_key,
                    early,
                    &AtomicBool::new(true),
                ),
                Err(DomError::Cancelled)
            );
            assert_eq!(staged.source_work.load(Ordering::Relaxed), spent);
            let limit = spent + 4096;
            let mut tight = generous;
            tight.work = limit;
            staged.source_limits = Some(tight);
            let result =
                staged.preflight_top_layer_adoption(subject_key, destination_key, early, &cancel);
            if let Err(error) = &result {
                assert_eq!(error, &DomError::SourceWorkLimit { limit });
            }
            let after = staged.source_work.load(Ordering::Relaxed);
            assert!(after >= spent);
            assert_eq!(staged.select_temporary_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(staged.node(subject_key).unwrap().document(), origin_key);
            assert_eq!(staged.node(subject_key).unwrap().parent(), None);
            staged.source_limits = Some(generous);
            // Same-subject qualification still succeeds under the original
            // generous policy; refusal never becomes a top-layer rejection.
            staged
                .preflight_top_layer_adoption(subject_key, destination_key, early, &cancel)
                .unwrap();
            results.push(result);
        }
        let checkpoint = staged
            .source_checkpoint(
                staged.source_other_bytes,
                staged.source_other_units,
                &cancel,
            )
            .unwrap();
        staged
            .adopt(subject_key, destination_key, dom.limits, &cancel)
            .unwrap();
        assert_eq!(
            staged.node(subject_key).unwrap().document(),
            destination_key
        );
        checkpoint.restore(&mut staged);
        assert_eq!(staged.node(subject_key).unwrap().document(), origin_key);
        assert_eq!(staged.node(subject_key).unwrap().parent(), None);
        let Some(transaction::InteractiveRead::Known {
            top_layer,
            pending_top_layer_removals,
            ..
        }) = staged.node(origin_key).unwrap().interactive_document()
        else {
            panic!("actual known old-Document relations")
        };
        assert_eq!(top_layer.iter().collect::<Vec<_>>(), keys);
        assert_eq!(pending_top_layer_removals.iter().collect::<Vec<_>>(), keys);
        assert_eq!(old.node(subject).unwrap().owner_document(), Some(origin));
        assert_eq!(old.node(subject).unwrap().parent(), None);
        assert_eq!(
            old.document(origin)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            members
        );
        assert_eq!(
            old.document(origin)
                .unwrap()
                .top_layer()
                .unwrap()
                .pending_removals(),
            members
        );
        assert!(
            old.document(destination)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members()
                .is_empty()
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        for result in results {
            assert_eq!(
                result,
                Ok(()),
                "one detached leaf adoption, unrelated Raw and Pending rows={unrelated}"
            );
        }
    }
}
