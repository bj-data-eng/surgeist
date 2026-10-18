//! Controller preparation and retry only; trusted native publications come from
//! the existing fixture. No Runtime Window, loop or cleanup proof is invented.
use super::*;
use crate::{DomSourceProgress, DomSourceRequest};
use std::panic::{AssertUnwindSafe, catch_unwind};
use surgeist_html_semantics::HtmlAboutBaseUrl;

const UNRELATED: usize = 128;
const PATH_ALLOWANCE: u64 = 96;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
struct Fixture {
    dom: Dom,
    source: DomHostedSource,
    creation: DomHostedCreation,
}
impl Fixture {
    fn new() -> Self {
        let (dom, mut source, document, _) = source::operation::fullscreen::native_fixture(
            DomLimits::new(4096, 1_000_000, 4096, 128, 128, 32, 128_000_000).unwrap(),
        );
        source
            .set_limits(
                DomHostedLimits::new(
                    DomHostedRecordLimits::new(1024, 1024, 1024, 1024, 1024, 1024, 1024),
                    256_000_000,
                    256_000_000,
                    100_000_000,
                ),
                &cancel(),
            )
            .unwrap();
        let navigable = dom
            .snapshot()
            .document(document)
            .unwrap()
            .navigable()
            .unwrap();
        let graph = dom.state.hosted_history.as_ref().unwrap();
        let nav = graph.navigable(navigable).unwrap();
        let creation = DomHostedCreation {
            document,
            host: nav.host,
            context: nav.context,
            global: nav.global,
            entry: nav.active,
            navigable,
        };
        Self {
            dom,
            source,
            creation,
        }
    }
    fn work(&self) -> Arc<Work> {
        self.source.work(&cancel()).unwrap()
    }
    fn install(
        &mut self,
        creation: DomHostedCreation,
        work: Arc<Work>,
        history: Option<DomHostedOperationId>,
    ) -> DomHostedOperationId {
        self.dom
            .install_hosted_invocation(
                &mut self.source,
                creation,
                creation.navigable,
                work,
                history,
                &cancel(),
            )
            .unwrap()
    }
    fn unrelated(&mut self, count: usize) -> Vec<DomHostedOperationId> {
        let creations: Vec<_> = (0..count)
            .map(|_| {
                // Real role issuance and Graph construction, matching the original
                // preparation fixture. These extra rows claim no Document activity.
                let prefix = self
                    .source
                    .prepare_top_level(self.creation.host, &cancel())
                    .unwrap();
                let work = prefix.work;
                let original = self
                    .source
                    .begin_operation(&work, 10, 0, &cancel())
                    .unwrap();
                let origin = self
                    .dom
                    .snapshot()
                    .document(self.creation.document)
                    .unwrap()
                    .metadata()
                    .origin()
                    .clone();
                let mut request = Request {
                    key: RequestKey {
                        operation: original,
                        phase: 0,
                    },
                    host: prefix.host,
                    context: prefix.context,
                    work: work.clone(),
                    parent: None,
                };
                advance(&self.source, &mut request);
                let inputs = crate::document::HostedDocumentInputs::form(
                    &self.source,
                    &work,
                    &origin,
                    &cancel(),
                )
                .unwrap();
                let execution = DomHostedExecutionRequest {
                    request,
                    origin: origin.clone(),
                    inputs,
                };
                let global = self.source.issue_global(&execution, &cancel()).unwrap();
                let mut request = execution.request;
                for _ in 0..3 {
                    advance(&self.source, &mut request);
                }
                let entry_request = DomHostedEntryRequest {
                    request,
                    document: self.creation.document,
                    global,
                };
                let entry = self.source.issue_entry(&entry_request, &cancel()).unwrap();
                let navigable = self
                    .source
                    .issue_navigable(&entry_request, &cancel())
                    .unwrap();
                let mut request = entry_request.request;
                advance(&self.source, &mut request);
                self.source
                    .permit(&request, self.source.revision().unwrap(), true, &cancel())
                    .unwrap()
                    .commit();
                let creation = DomHostedCreation {
                    document: self.creation.document,
                    host: self.creation.host,
                    context: request.context,
                    global,
                    entry,
                    navigable,
                };
                let key = uuid();
                let name = DocumentString::from("");
                let about = HtmlAboutBaseUrl::KnownNull;
                let context_cancel = cancel();
                let context = history::Context::new(&self.source, &work, &context_cancel, 0);
                let graph = history::Graph::initial(
                    self.dom.state.hosted_history.as_deref(),
                    history::Initial {
                        creation,
                        initialization: DomHostedEntryInitialization::new(key, key),
                        origin,
                        parent: None,
                        target_name: Some(&name),
                        about_base_url: &about,
                        initiator_origin: None,
                    },
                    &context,
                )
                .unwrap();
                let mut draft = Draft::new(&self.dom, DomBatch::new().owner, &cancel()).unwrap();
                draft
                    .set_hosted_history(Some(graph), None, &cancel())
                    .unwrap();
                let prepared = draft.finish(&self.dom, &cancel()).unwrap();
                self.dom.publish(prepared, &cancel()).unwrap();
                creation
            })
            .collect();
        // Finish each synchronous constructor before retaining any independent
        // parallel invocation Work; prepare_top_level correctly forbids overlap.
        creations
            .into_iter()
            .map(|creation| {
                let work = self.work();
                self.install(creation, work, None)
            })
            .collect()
    }
    fn retire_rows(&mut self, rows: &[DomHostedOperationId]) {
        for id in rows {
            self.dom.hosted_operations.remove(*id).unwrap();
            self.source
                .retire(
                    Role::Operation,
                    id.source,
                    id.slot,
                    id.generation,
                    &cancel(),
                )
                .unwrap();
        }
    }
    fn queued_history(&mut self, work: Arc<Work>) -> DomHostedOperationId {
        let slot = self.source.next_operation_slot(&work, &cancel()).unwrap();
        let bank = self
            .dom
            .hosted_operations
            .prepare(slot, &work, &self.source, 0, None, &cancel())
            .unwrap();
        let retained = self
            .source
            .retain(size_of::<Operation>(), &cancel())
            .unwrap();
        let id = self
            .source
            .begin_operation(&work, 2, bank.scratch_bytes().unwrap(), &cancel())
            .unwrap();
        let request = Request {
            key: RequestKey {
                operation: id,
                phase: 0,
            },
            host: self.creation.host,
            context: self.creation.context,
            work,
            parent: None,
        };
        let origin = self
            .dom
            .snapshot()
            .document(self.creation.document)
            .unwrap()
            .metadata()
            .origin()
            .clone();
        self.dom.hosted_operations.insert_prepared(
            bank,
            Operation {
                origin,
                policy: None,
                phase: CreationPhase::Queued(queue::Continuation::removal(
                    request,
                    self.creation,
                    self.creation.navigable,
                )),
                _retained: retained,
            },
            false,
        );
        id
    }
}
fn uuid() -> [u16; 36] {
    "00000000-0000-4000-8000-000000000000"
        .encode_utf16()
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
fn advance(source: &DomHostedSource, request: &mut Request) {
    source
        .permit(request, source.revision().unwrap(), false, &cancel())
        .unwrap()
        .commit();
    request.key.phase += 1;
}
fn phase(f: &Fixture, id: DomHostedOperationId) -> &Invocation {
    let CreationPhase::Invocation(row) = &f.dom.hosted_operations.get(id).unwrap().phase else {
        panic!("original invocation")
    };
    row
}

#[test]
fn exact_invocation_retry_keeps_original_work_and_phase_without_unrelated_row_discovery() {
    fn sample(count: usize, retired: bool) -> u64 {
        let mut f = Fixture::new();
        let unrelated = f.unrelated(count);
        if retired {
            f.retire_rows(&unrelated);
        }
        let work = f.work();
        let id = f.install(f.creation, work.clone(), None);
        let old = f.dom.snapshot();
        let graph = f.dom.state.hosted_history.clone().unwrap();
        let used = f.source.retained_storage_bytes();
        let limits = f.source.limits();
        f.source
            .set_limits(
                DomHostedLimits::new(
                    limits.records(),
                    used,
                    limits.scratch_bytes(),
                    limits.work(),
                ),
                &cancel(),
            )
            .unwrap();
        let before = work.spent.load(Ordering::Relaxed);
        let result = f.dom.begin_hosted_invocation(
            &mut f.source,
            f.creation,
            f.creation.navigable,
            work.clone(),
            None,
            &cancel(),
        );
        let steps = work.spent.load(Ordering::Relaxed) - before;
        assert!(
            matches!(
                &result,
                Err(DomHostedError::Dom(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: 0,
                }))
            ),
            "real retained-prefix refusal: {:?}",
            result.err()
        );
        assert!(Arc::ptr_eq(&phase(&f, id).request.request.work, &work));
        assert_eq!(phase(&f, id).request.request.key.operation, id);
        assert_eq!(phase(&f, id).request.request.key.phase, 0);
        assert!(phase(&f, id).history == HistoryDependency::None);
        assert_eq!(phase(&f, id).request.cleanup, None);
        assert!(Arc::ptr_eq(
            &graph,
            f.dom.state.hosted_history.as_ref().unwrap()
        ));
        assert_eq!(f.dom.snapshot().stamp(), old.stamp());
        f.source.set_limits(limits, &cancel()).unwrap();
        let retried = f
            .dom
            .begin_hosted_invocation(
                &mut f.source,
                f.creation,
                f.creation.navigable,
                work.clone(),
                None,
                &cancel(),
            )
            .unwrap();
        assert_eq!(retried.operation, id);
        assert!(Arc::ptr_eq(&phase(&f, id).request.request.work, &work));
        assert_eq!(phase(&f, id).request.request.key.phase, 1);
        assert!(phase(&f, id).request.cleanup.is_some());
        assert!(
            !old.state
                .hosted_history
                .as_ref()
                .unwrap()
                .navigable(f.creation.navigable)
                .unwrap()
                .retiring
        );
        assert!(
            f.dom
                .state
                .hosted_history
                .as_ref()
                .unwrap()
                .navigable(f.creation.navigable)
                .unwrap()
                .retiring
        );
        steps
    }
    let small = sample(0, false);
    for retired in [false, true] {
        let large = sample(UNRELATED, retired);
        assert!(
            large <= small + PATH_ALLOWANCE,
            "retry discovery across {UNRELATED} rows (retired={retired}): small={small}, large={large}"
        );
    }
}

#[test]
fn pending_history_lookup_and_absence_ignore_unrelated_parked_and_vacant_rows() {
    fn sample(count: usize, retired: bool) -> (u64, u64) {
        let mut f = Fixture::new();
        let unrelated = f.unrelated(count);
        if retired {
            f.retire_rows(&unrelated);
        }
        let work = f.work();
        let history = f.queued_history(work.clone());
        let id = f.install(f.creation, work.clone(), Some(history));
        let missing = f.install(f.creation, work.clone(), None);
        let before = work.spent.load(Ordering::Relaxed);
        assert_eq!(
            f.dom
                .prepare_hosted_history_completion(history, &work, &cancel())
                .unwrap(),
            id
        );
        let found = work.spent.load(Ordering::Relaxed) - before;
        let before = work.spent.load(Ordering::Relaxed);
        assert_eq!(
            f.dom
                .prepare_hosted_history_completion(missing, &work, &cancel()),
            Err(DomHostedError::WrongPhase)
        );
        let absent = work.spent.load(Ordering::Relaxed) - before;
        f.dom.settle_hosted_history_completion(id, history);
        assert!(phase(&f, id).history == HistoryDependency::Settled(history));
        assert_eq!(
            f.dom
                .prepare_hosted_history_completion(history, &work, &cancel()),
            Err(DomHostedError::WrongPhase)
        );
        assert!(Arc::ptr_eq(&phase(&f, id).request.request.work, &work));
        (found, absent)
    }
    let small = sample(0, false);
    for retired in [false, true] {
        let large = sample(UNRELATED, retired);
        assert!(
            large.0 <= small.0 + PATH_ALLOWANCE && large.1 <= small.1 + PATH_ALLOWANCE,
            "exact history link/absence across {UNRELATED} rows (retired={retired}): small={small:?}, large={large:?}"
        );
    }
}

#[test]
fn cleanup_reuses_the_original_document_global_row_without_discovering_other_operations() {
    fn sample(count: usize) -> u64 {
        let mut f = Fixture::new();
        f.unrelated(count);
        let work = f.work();
        let first = f
            .dom
            .begin_hosted_invocation(
                &mut f.source,
                f.creation,
                f.creation.navigable,
                work.clone(),
                None,
                &cancel(),
            )
            .unwrap();
        let cleanup = phase(&f, first.operation).request.cleanup.unwrap();
        let second = f.install(f.creation, work.clone(), None);
        let revision = f.source.revision().unwrap();
        let before = work.spent.load(Ordering::Relaxed);
        let completed = f
            .dom
            .complete_hosted_invocation(&mut f.source, second, &cancel())
            .unwrap();
        let steps = work.spent.load(Ordering::Relaxed) - before;
        assert_eq!(completed.operation, second);
        assert_eq!(phase(&f, second).request.cleanup, Some(cleanup));
        assert_eq!(phase(&f, first.operation).request.cleanup, Some(cleanup));
        assert_eq!(f.source.revision().unwrap(), revision);
        let CreationPhase::DestroyEnqueue(request) =
            &f.dom.hosted_operations.get(cleanup).unwrap().phase
        else {
            panic!("original pending cleanup")
        };
        assert_eq!(request.creation.document, f.creation.document);
        assert_eq!(request.creation.global, f.creation.global);
        steps
    }
    let small = sample(0);
    let large = sample(UNRELATED);
    assert!(
        large <= small + PATH_ALLOWANCE,
        "cleanup relationship discovery: small={small}, large={large}"
    );
}

#[test]
fn empty_preparation_ignores_historical_rows_and_borrowed_restore_is_infallible() {
    fn sample(count: usize) -> u64 {
        let mut f = Fixture::new();
        let unrelated = f.unrelated(count);
        f.retire_rows(&unrelated);
        let work = f.work();
        let id = f.install(f.creation, work.clone(), None);
        let before = work.spent.load(Ordering::Relaxed);
        let source = &f.source;
        let result: Result<(), DomHostedError> =
            f.dom.with_hosted_operation(id, |dom, operation| {
                assert!(Arc::ptr_eq(&operation.request().work, &work));
                let prepared =
                    dom.hosted_operations
                        .prepare(id.slot, &work, source, 0, None, &cancel())?;
                assert_eq!(prepared.backing_bytes().unwrap(), 0);
                drop(prepared);
                Err(DomError::Cancelled.into())
            });
        let steps = work.spent.load(Ordering::Relaxed) - before;
        assert_eq!(result, Err(DomError::Cancelled.into()));
        assert!(Arc::ptr_eq(&phase(&f, id).request.request.work, &work));
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _: Result<(), DomHostedError> = f
                    .dom
                    .with_hosted_operation(id, |_, _| panic!("intentional owning-guard unwind"));
            }))
            .is_err()
        );
        assert_eq!(phase(&f, id).request.request.key.operation, id);
        assert!(Arc::ptr_eq(&phase(&f, id).request.request.work, &work));
        steps
    }
    let small = sample(0);
    let large = sample(UNRELATED);
    assert!(
        large <= small + PATH_ALLOWANCE,
        "empty slot preparation: small={small}, large={large}"
    );
}

fn source_limits() -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(128_000_000, 4_000_000, 100_000_000, 128, 4096, 4096, 4)
}
fn insert_absent_src_frame(f: &mut Fixture) -> (NodeId, DomHostedCreation) {
    use crate::{
        DomHostedChildStep as Step, DomInitialPermissionsPolicy, DomPermissionsPolicyValue,
        DomSourceEdit,
    };
    let parent = f
        .dom
        .snapshot()
        .document(f.creation.document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(f.creation.document, None);
    let ticket = batch.create_element_ns(&closed, crate::Namespace::html(), "frame");
    let frame = f
        .dom
        .apply(batch, &cancel())
        .unwrap()
        .created(&ticket)
        .unwrap()
        .unwrap();
    let mut lease = f
        .dom
        .begin_source_edit(
            DomSourceEdit::insert_before(frame, parent, None),
            source_limits(),
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("actual Frame construction request")
    };
    let preparation = lease
        .prepare_hosted_child(&mut f.source, &child, &cancel())
        .unwrap();
    let mut prepared = Some(
        f.source
            .seal_child_preparation(&child, &mut Some(preparation), &cancel())
            .unwrap(),
    );
    lease
        .accept_hosted_child_preparation(&mut f.source, &child, &mut prepared, &cancel())
        .unwrap();
    assert!(prepared.is_none());
    let mut creation = None;
    for _ in 0..12 {
        let publication = match lease.hosted_child_step(&child).unwrap() {
            Step::PermissionsPolicy(request) => {
                let mut policy = Some(
                    f.source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel(),
                        )
                        .unwrap(),
                );
                lease
                    .accept_hosted_child_policy(&mut f.source, &child, &mut policy, &cancel())
                    .unwrap();
                continue;
            }
            Step::NativeExecution(request) => {
                let global = f.source.issue_global(request, &cancel()).unwrap();
                f.source
                    .seal_native_execution(request, global, 200, &cancel())
                    .unwrap()
            }
            Step::MakeActive(request) => f.source.seal_make_active(request, &cancel()).unwrap(),
            Step::FinishLoading(request) => f
                .source
                .seal_finish_loading(request, 300, &cancel())
                .unwrap(),
            Step::InitialEntry(request) => {
                let entry = f.source.issue_entry(request, &cancel()).unwrap();
                let navigable = f.source.issue_navigable(request, &cancel()).unwrap();
                f.source
                    .seal_initial_entry(
                        request,
                        entry,
                        navigable,
                        &mut Some(DomHostedEntryInitialization::new(uuid(), uuid())),
                        &cancel(),
                    )
                    .unwrap()
            }
            Step::QueueHistory(request) => {
                creation = Some(request.creation());
                let mut publication =
                    Some(f.source.seal_child_history(request, &cancel()).unwrap());
                lease
                    .accept_hosted_child_history(&mut f.source, &child, &mut publication, &cancel())
                    .unwrap();
                assert!(publication.is_none());
                continue;
            }
            Step::NeedsUrl(_) => {
                // The supported absent-src branch compares real about:blank
                // inputs against its inclusive ancestor and returns null.
                lease
                    .process_hosted_child_attributes(&child, &cancel())
                    .unwrap();
                assert!(matches!(
                    lease.hosted_child_step(&child).unwrap(),
                    Step::Returned
                ));
                break;
            }
            _ => panic!("unexpected constructor phase"),
        };
        lease
            .accept_hosted_child_publication(
                &mut f.source,
                &child,
                &mut Some(publication),
                &cancel(),
            )
            .unwrap();
    }
    let creation = creation.expect("authentic initial entry and queued history");
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(
        f.dom
            .snapshot()
            .node(frame)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(Some(creation.navigable))
    );
    (frame, creation)
}

#[test]
fn source_removal_handoff_preserves_receipt_history_and_original_work_across_unrelated_operations()
{
    use crate::{DomHostedChildStep, DomSourceEdit};
    fn sample(count: usize) -> u64 {
        let mut f = Fixture::new();
        f.unrelated(count);
        let (frame, creation) = insert_absent_src_frame(&mut f);
        let old = f.dom.snapshot();
        let mut lease = f
            .dom
            .begin_source_edit(DomSourceEdit::detach(frame), source_limits(), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual Frame removal request")
        };
        lease
            .prepare_hosted_child_removal(&mut f.source, &child, &cancel())
            .unwrap();
        let DomHostedChildStep::Removal(request) = lease.hosted_child_step(&child).unwrap() else {
            panic!("original removal packet")
        };
        assert_eq!(request.creation(), creation);
        let history_id = request.operation();
        let original_work = request.request.work.clone();
        let mut publication = Some(f.source.seal_child_removal(request, &cancel()).unwrap());
        let before = lease.snapshot();
        let revision = f.source.revision().unwrap();
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            lease.accept_hosted_child_removal(&mut f.source, &child, &mut publication, &cancelled),
            Err(DomHostedError::Dom(DomError::Cancelled))
        ));
        assert!(publication.is_some());
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert_eq!(f.source.revision().unwrap(), revision);
        let before = original_work.spent.load(Ordering::Relaxed);
        let handoff = lease
            .accept_hosted_child_removal(&mut f.source, &child, &mut publication, &cancel())
            .unwrap();
        let steps = original_work.spent.load(Ordering::Relaxed) - before;
        let (invocation, history) = handoff.into_parts();
        assert_eq!(history.operation(), history_id);
        assert!(publication.is_none());
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        drop(lease);
        let CreationPhase::Invocation(row) = &f
            .dom
            .hosted_operations
            .get(invocation.operation)
            .unwrap()
            .phase
        else {
            panic!("retained original invocation")
        };
        assert_eq!(row.request.creation, creation);
        assert!(row.history == HistoryDependency::Pending(history_id));
        assert!(Arc::ptr_eq(&row.request.request.work, &original_work));
        assert!(row.request.cleanup.is_some());
        assert_eq!(
            f.dom
                .prepare_hosted_history_completion(history_id, &original_work, &cancel())
                .unwrap(),
            invocation.operation
        );
        assert_eq!(
            old.node(frame)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(Some(creation.navigable))
        );
        assert_eq!(
            f.dom
                .snapshot()
                .node(frame)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
        assert!(
            f.dom
                .state
                .hosted_history
                .as_ref()
                .unwrap()
                .navigable(creation.navigable)
                .unwrap()
                .retiring
        );
        steps
    }
    let small = sample(0);
    let large = sample(UNRELATED);
    assert!(
        large <= small + 192,
        "real Source removal handoff across {UNRELATED} unrelated operations: small={small}, large={large}"
    );
}

#[test]
fn operation_lookup_rejects_wrong_generation_source_and_reused_identity() {
    let mut f = Fixture::new();
    let work = f.work();
    let original = f.install(f.creation, work.clone(), None);
    for generation in [original.generation + 1, 1 << 63, u64::MAX] {
        let wrong_generation = DomHostedOperationId {
            generation,
            ..original
        };
        assert!(matches!(
            f.dom.hosted_operations.get(wrong_generation),
            Err(DomHostedError::StaleIdentity)
        ));
    }
    let wrong_serial = DomHostedOperationId {
        source: DomHostedSourceId {
            serial: original.source.serial + 1,
            ..original.source
        },
        ..original
    };
    assert!(matches!(
        f.dom.hosted_operations.get(wrong_serial),
        Err(DomHostedError::StaleIdentity)
    ));
    let other = Fixture::new();
    let foreign = DomHostedOperationId {
        source: other.source.id(),
        ..original
    };
    assert!(matches!(
        f.dom.hosted_operation(foreign),
        Err(DomHostedError::ForeignSource)
    ));
    f.retire_rows(&[original]);
    let reused = f.install(f.creation, work.clone(), None);
    assert_eq!(reused.slot, original.slot);
    assert_eq!(reused.generation, original.generation + 1);
    assert!(matches!(
        f.dom.hosted_operation(original),
        Err(DomHostedError::StaleIdentity)
    ));
    assert!(Arc::ptr_eq(
        &f.dom.hosted_operation(reused).unwrap().request().work,
        &work
    ));
}
