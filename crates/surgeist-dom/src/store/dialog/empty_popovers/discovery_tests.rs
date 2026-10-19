//! Real inactive Show receives its actual native event execution receipt.
use super::*;
use std::cell::Cell;
#[derive(Clone, Copy, Default, Debug)]
struct Reads {
    calls: usize,
    work: u64,
}
thread_local! { static READS: Cell<Option<Reads>> = const { Cell::new(None) }; }
pub(super) struct QueryRead<'a> {
    draft: &'a Draft,
    start: u64,
}
impl<'a> QueryRead<'a> {
    pub(super) fn begin(draft: &'a Draft) -> Self {
        Self {
            draft,
            start: draft.source_work.load(Ordering::Relaxed),
        }
    }
}
impl Drop for QueryRead<'_> {
    fn drop(&mut self) {
        if let Some(mut reads) = READS.get() {
            reads.calls += 1;
            reads.work += self.draft.source_work.load(Ordering::Relaxed) - self.start;
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
fn policy() -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(256_000_000, 4_000_000, 100_000_000, 64, 4096, 4096, 2)
}

// Adequate fixture configuration, not an operation-cost oracle: existing owner
// storage is already controlled, and the original fixed 256 MB is additional
// preparation/old-new overlap headroom. Work and discovery assertions remain
// independent of these capacity-based storage inputs.
fn owner_preparation_policy(dom: &Dom) -> crate::DomSourceLimits {
    let baseline = dom.custody_registry.ledger().total().bytes();
    let mut configured = policy();
    let additional = configured.retained_bytes;
    configured.retained_bytes = baseline
        .checked_add(additional)
        .expect("fixture owner baseline and preparation allowance");
    eprintln!(
        "discovery fixture storage: preoperation_owner={baseline}, additional={additional}, total={}",
        configured.retained_bytes
    );
    configured
}

#[test]
#[ignore = "requires an instrumented DOM build; observes actual Show popover queries"]
fn actual_show_proves_empty_popover_lists_without_visiting_hidden_raw_members() {
    scale_metrics::require_enabled().expect("compile with SURGEIST_DOM_STORAGE_COUNTERS=1");
    let cancel = AtomicBool::new(false);
    for unrelated in [0, 256, 4096] {
        let mut dom = Dom::new(
            DomLimits::new(
                8192,
                1_000_000,
                16_384,
                128,
                128,
                16,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_dispatch_custody(
                crate::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000)
                    .unwrap(),
            ),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let dialog = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        batch.insert_before(&dialog, &document, None);
        let tickets: Vec<_> = (0..unrelated)
            .map(|_| {
                let node = batch.create_element_ns(&document, crate::Namespace::html(), "div");
                batch.set_attribute(&node, "popover", "auto");
                node
            })
            .collect();
        let setup = dom.apply(batch, &cancel).unwrap();
        let document = setup.document(&document).unwrap();
        let dialog = setup.created(&dialog).unwrap().unwrap();
        let members: Vec<_> = tickets
            .iter()
            .map(|ticket| setup.created(ticket).unwrap().unwrap())
            .collect();
        // Population establishes canonical native Raw roles only. It does not
        // claim to execute a public Source command or measure its preparation.
        // The actual Show below retains the finite Source policy and receipts.
        let mut staged = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        for member in &members {
            let key = staged.resolve((*member).into()).unwrap();
            // Real raw relation owner, not a claim these Elements were shown.
            staged.add_top_layer(key, &cancel).unwrap();
        }
        dom.publish(staged.finish(&dom, &cancel).unwrap(), &cancel)
            .unwrap();
        let old = dom.snapshot();
        let executor = dom.bind_source_event_executor(&cancel).unwrap();
        let mut host_source = dom
            .register_hosted_source(
                crate::DomHostedLimits::new(
                    crate::DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    16_000_000,
                    16_000_000,
                    10_000_000,
                ),
                &cancel,
            )
            .unwrap();
        // This fresh trusted native Host has never owned a focused element.
        // It supplies only complete-Host absence, not a Window or focus effect.
        let host = host_source.issue_host(&cancel).unwrap();
        let tasks = dom.bind_element_task_source(&cancel).unwrap();
        let task_storage = tasks
            .retain_payload(&mut dom, size_of::<crate::DomDialogTaskInfo>(), &cancel)
            .unwrap();
        let mut queue = Vec::new();
        queue.try_reserve_exact(1).unwrap();
        assert_eq!(queue.capacity(), 1);
        let command_policy = owner_preparation_policy(&dom);
        let event = {
            let mut lease = dom
                .begin_source_edit(
                    crate::DomSourceEdit::show_dialog(dialog),
                    command_policy,
                    &cancel,
                )
                .unwrap();
            lease
                .bind_execution_host(&host_source, host, &cancel)
                .unwrap();
            let crate::DomSourceProgress::Request(crate::DomSourceRequest::Event(event)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("actual beforetoggle")
            };
            assert_eq!(event.kind(), crate::DomSourceEventKind::BeforeToggle);
            event
        };
        let mut permit = Some(executor.begin_dispatch(&dom, &event, &cancel).unwrap());
        let scope = executor
            .enter_dispatch(&dom, permit.as_ref().unwrap(), &cancel)
            .unwrap();
        drop(scope); // Truthful native dispatch with no callback mutation.
        let mut publication = Some(
            executor
                .seal_completed(
                    &dom,
                    &mut permit,
                    crate::DomSourceEventOutcome::NotCanceled,
                    &cancel,
                )
                .unwrap(),
        );
        let mut lease = dom
            .resume_source_edit(event.operation(), command_policy, &cancel)
            .unwrap();
        lease
            .accept_source_event(&event, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::Dialog(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("real QueueToggle boundary")
        };
        assert!(matches!(
            request.action(),
            surgeist_html::HtmlDialogAction::QueueToggle { .. }
        ));
        let plan = lease
            .dialog_task_queue_plan(&request, &tasks, 1, &cancel)
            .unwrap();
        assert!(plan.replacing().is_none());
        let fence = lease
            .prepare_dialog_task_queue(&request, &tasks, plan, &cancel)
            .unwrap();
        queue.push(plan.task()); // Actual admitted native queue installation.
        let (permission, publication) = fence.publish();
        let mut publication = Some(publication);
        assert_eq!(permission.info(), queue[0]);
        lease
            .accept_dialog_task_queue(&request, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        let queued_stamp = lease.snapshot().stamp();
        let spent = lease.status().spent_work();
        assert_eq!(
            lease.advance(&AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(lease.status().spent_work(), spent);
        assert_eq!(lease.snapshot().stamp(), queued_stamp);
        assert_eq!(queue[0], permission.info());
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::Dialog(remember)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual remembered-focus request after AddOpen")
        };
        assert_eq!(
            remember.action(),
            surgeist_html::HtmlDialogAction::RememberFocusedElement
        );
        let focus_request = crate::DomFocusRequest::RememberDialog(remember);
        let preparation = lease
            .prepare_focus(&focus_request, &host_source, &cancel)
            .unwrap()
            .expect("bound genuine native Host");
        assert_eq!(
            preparation.target(),
            crate::DomFocusTarget::RememberedHost(host)
        );
        let mut focus_publication = Some(
            preparation
                .publish(crate::DomFocusObservation::KnownNone)
                .unwrap(),
        );
        lease
            .accept_focus(
                &focus_request,
                &host_source,
                &mut focus_publication,
                &cancel,
            )
            .unwrap();
        assert!(focus_publication.is_none());
        let observation = Observation::begin();
        let progress = lease.advance(&cancel).unwrap();
        let reads = observation.finish();
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::DialogFocusing(request)) =
            progress
        else {
            panic!("actual unavailable allow-focus boundary, no fabricated focus reply")
        };
        assert_eq!(
            request.action(),
            surgeist_html::HtmlDialogFocusingAction::ReadAllowFocus
        );
        assert_eq!(
            lease
                .snapshot()
                .node(dialog)
                .unwrap()
                .attribute(&"open".into()),
            Some(&DocumentString::from(""))
        );
        assert_eq!(old.node(dialog).unwrap().attribute(&"open".into()), None);
        let pending = lease.snapshot();
        assert_eq!(
            pending.document(document).unwrap().top_layer().unwrap_err(),
            DomError::IncompleteSourceOperation
        );
        let node = pending.state.node(pending.owner, document.node()).unwrap();
        let Some(canonical::Interactive::Known { top_layer, .. }) = &node.interactive_document
        else {
            panic!("actual known canonical raw list")
        };
        assert_eq!(top_layer.as_slice(), members);
        assert!(
            reads.calls > 0,
            "real Show must reach its current/captured Document queries"
        );
        assert!(
            reads.work <= 64,
            "showing participants remain zero, unrelated={unrelated}: {reads:?}"
        );
        drop(lease);
        drop(permission);
        drop(queue); // Actual queue storage is released before its admission.
        drop(task_storage);
    }
}

#[test]
fn actual_source_adoption_keeps_original_raw_owner_and_current_empty_popover_facts_distinct() {
    let cancel = AtomicBool::new(false);
    for unrelated in [0, 256] {
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
        let first = batch.create_document();
        let second = batch.create_document();
        let dialog = batch.create_element_ns(&first, crate::Namespace::html(), "dialog");
        batch.insert_before(&dialog, &first, None);
        let tickets: Vec<_> = (0..unrelated)
            .map(|_| batch.create_element_ns(&first, crate::Namespace::html(), "div"))
            .collect();
        let setup = dom.apply(batch, &cancel).unwrap();
        let first = setup.document(&first).unwrap();
        let second = setup.document(&second).unwrap();
        let dialog = setup.created(&dialog).unwrap().unwrap();
        let members: Vec<_> = tickets
            .iter()
            .map(|ticket| setup.created(ticket).unwrap().unwrap())
            .collect();
        let batch = DomBatch::new();
        let (mut staged, _) = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            policy(),
            &cancel,
        )
        .unwrap();
        for member in members.iter().chain(std::iter::once(&dialog)) {
            let key = staged.resolve((*member).into()).unwrap();
            staged.add_top_layer(key, &cancel).unwrap();
        }
        dom.publish(staged.finish(&dom, &cancel).unwrap(), &cancel)
            .unwrap();
        let old = dom.snapshot();
        let mut lease = dom
            .begin_source_edit(
                crate::DomSourceEdit::adopt(dialog, second.node()),
                policy(),
                &cancel,
            )
            .unwrap();
        let operation = lease.operation();
        assert_eq!(
            lease.advance(&AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(lease.snapshot().stamp(), old.stamp());
        let crate::DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("actual local adoption has no fabricated external reply")
        };
        assert_eq!(done.operation(), operation);
        drop(lease);
        let adopted = dom.snapshot();
        assert_eq!(adopted.node(dialog).unwrap().owner_document(), Some(second));
        assert_eq!(
            adopted
                .document(first)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            members
        );
        assert!(
            adopted
                .document(second)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members()
                .is_empty()
        );
        assert_eq!(old.node(dialog).unwrap().owner_document(), Some(first));
        assert_eq!(
            old.document(first)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members()
                .last(),
            Some(&dialog)
        );
        let batch = DomBatch::new();
        let (mut staged, _) = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            policy(),
            &cancel,
        )
        .unwrap();
        let spent = staged.source_work.load(Ordering::Relaxed);
        let mut tight = policy();
        tight.work = spent + 64;
        staged.source_limits = Some(tight);
        let original = staged.document_known_empty_popovers(first, &cancel);
        staged.source_limits = Some(policy());
        assert!(
            staged
                .document_known_empty_popovers(first, &cancel)
                .unwrap()
        );
        assert!(
            staged
                .document_known_empty_popovers(second, &cancel)
                .unwrap()
        );
        assert_eq!(dom.snapshot().stamp(), adopted.stamp());
        assert_eq!(
            original,
            Ok(true),
            "original owner retains only hidden unrelated members={unrelated}"
        );
    }
}
