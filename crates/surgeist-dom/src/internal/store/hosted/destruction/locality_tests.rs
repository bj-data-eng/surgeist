//! Controller preparation and retry only; trusted native publications come from
//! the existing fixture. No Runtime Window, loop or cleanup proof is invented.
use super::*;
use crate::internal::store::DomWindowNamedValue;
use crate::{internal::source::DomSourceProgress, internal::source::DomSourceRequest};
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
            DomLimits::new(
                4096,
                1_000_000,
                4096,
                128,
                128,
                32,
                128_000_000,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
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
                let inputs = crate::internal::document::HostedDocumentInputs::form(
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
                        insertion: None,
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
        let retained = self
            .source
            .retain(size_of::<Operation>(), &cancel())
            .unwrap();
        let bank = self
            .dom
            .hosted_operations
            .prepare(
                slot,
                &work,
                &self.source,
                0,
                None,
                operations::BodyCredit::Owned(retained),
                operations::Projection::default(),
                &cancel(),
            )
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
        let (bank, retained) = bank.into_owned();
        drop(self.dom.hosted_operations.insert_prepared(
            operations::Transfer::new(
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
            ),
            false,
        ));
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
        assert_eq!(
            old.document(f.creation.document)
                .unwrap()
                .focused_area_designation(),
            crate::internal::document::DomFocusedAreaDesignation::KnownNull
        );
        assert_eq!(
            f.dom
                .snapshot()
                .document(f.creation.document)
                .unwrap()
                .focused_area_designation(),
            crate::internal::document::DomFocusedAreaDesignation::KnownNull
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
                let prepared = dom.hosted_operations.prepare(
                    id.slot,
                    &work,
                    source,
                    0,
                    None,
                    operations::BodyCredit::Borrowed(&operation._retained),
                    operations::Projection::default(),
                    &cancel(),
                )?;
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

fn source_limits() -> crate::internal::source::DomSourceLimits {
    crate::internal::source::DomSourceLimits::new(
        128_000_000,
        4_000_000,
        100_000_000,
        128,
        4096,
        4096,
        4,
    )
}
fn insert_absent_src_frame(f: &mut Fixture) -> (NodeId, DomHostedCreation) {
    insert_absent_src_element(f, "frame")
}
fn insert_absent_src_element(f: &mut Fixture, local: &str) -> (NodeId, DomHostedCreation) {
    let parent = f
        .dom
        .snapshot()
        .document(f.creation.document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(f.creation.document, None);
    let ticket = batch.create_element_ns(&closed, surgeist_html::values::Namespace::html(), local);
    let frame = f
        .dom
        .apply(batch, &cancel())
        .unwrap()
        .created(&ticket)
        .unwrap()
        .unwrap();
    let [creation] = drive_native_insertion(f, frame, parent, None);
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

fn drive_absent_src_child(
    lease: &mut source::operation::DomSourceLease<'_>,
    source: &mut DomHostedSource,
    child: &DomHostedChildRequest,
) -> DomHostedCreation {
    use crate::{
        internal::hosted::DomHostedChildStep as Step,
        internal::hosted::DomInitialPermissionsPolicy, internal::hosted::DomPermissionsPolicyValue,
    };
    let preparation = lease
        .prepare_hosted_child(source, child, &cancel())
        .unwrap();
    let mut prepared = Some(
        source
            .seal_child_preparation(child, &mut Some(preparation), &cancel())
            .unwrap(),
    );
    lease
        .accept_hosted_child_preparation(source, child, &mut prepared, &cancel())
        .unwrap();
    assert!(prepared.is_none());
    let mut creation = None;
    for _ in 0..12 {
        let publication = match lease.hosted_child_step(child).unwrap() {
            Step::PermissionsPolicy(request) => {
                let mut policy = Some(
                    source
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
                    .accept_hosted_child_policy(source, child, &mut policy, &cancel())
                    .unwrap();
                continue;
            }
            Step::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel()).unwrap();
                source
                    .seal_native_execution(request, global, 200, &cancel())
                    .unwrap()
            }
            Step::MakeActive(request) => source.seal_make_active(request, &cancel()).unwrap(),
            Step::FinishLoading(request) => {
                source.seal_finish_loading(request, 300, &cancel()).unwrap()
            }
            Step::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel()).unwrap();
                let navigable = source.issue_navigable(request, &cancel()).unwrap();
                source
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
                let mut publication = Some(source.seal_child_history(request, &cancel()).unwrap());
                lease
                    .accept_hosted_child_history(source, child, &mut publication, &cancel())
                    .unwrap();
                assert!(publication.is_none());
                continue;
            }
            Step::NeedsUrl(_) => {
                // The supported absent-src branch compares real about:blank
                // inputs against its inclusive ancestor and returns null.
                lease
                    .process_hosted_child_attributes(child, &cancel())
                    .unwrap();
                assert!(matches!(
                    lease.hosted_child_step(child).unwrap(),
                    Step::Returned
                ));
                break;
            }
            _ => panic!("unexpected constructor phase"),
        };
        lease
            .accept_hosted_child_publication(source, child, &mut Some(publication), &cancel())
            .unwrap();
    }
    creation.expect("authentic initial entry and queued history")
}

#[test]
fn source_removal_handoff_preserves_receipt_history_and_original_work_across_unrelated_operations()
{
    use crate::{internal::hosted::DomHostedChildStep, internal::source::DomSourceEdit};
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

/// These are the existing owning controller/issuer APIs, not an attestation of
/// Window cleanup or public navigation completion. Every row has issued
/// identities; the storage owner is explicitly retired by its original helper.
fn retained_controller_cycles(complete: bool, narrow: bool) {
    let mut f = Fixture::new();
    let mut baseline = None;
    let mut slots = None;
    let mut last_history = None;
    for cycle in 0..8 {
        let work = f.work();
        let history = f.queued_history(work.clone());
        let invocation = f.install(f.creation, work.clone(), Some(history));
        if let Some((history_slot, invocation_slot)) = slots {
            assert_eq!(
                (history.slot, invocation.slot),
                (history_slot, invocation_slot)
            );
        } else {
            slots = Some((history.slot, invocation.slot));
        }
        if let Some(previous) = last_history {
            assert!(history.generation > previous);
        }
        last_history = Some(history.generation);
        let before = f.source.retained_storage_bytes();
        let physical = f.dom.hosted_operations.inspect_storage_bytes();
        // Temporary take/restore is distinct: it must retain the exact original
        // row and every admitted path even when no query may see the held row.
        f.dom
            .with_hosted_operation(invocation, |_, operation| {
                assert_eq!(operation.request().key.operation, invocation);
                assert!(Arc::ptr_eq(&operation.request().work, &work));
                Ok(OperationResult::Retain(()))
            })
            .unwrap();
        assert_eq!(f.source.retained_storage_bytes(), before);
        assert_eq!(f.dom.hosted_operations.inspect_storage_bytes(), physical);
        assert!(Arc::ptr_eq(
            &phase(&f, invocation).request.request.work,
            &work
        ));
        let cleanup = if narrow {
            let source = &mut f.source;
            Some(
                f.dom
                    .with_hosted_operation(invocation, |dom, operation| {
                        let CreationPhase::Invocation(row) = &operation.phase else {
                            panic!("actual original invocation")
                        };
                        let cleanup = dom.install_hosted_cleanup(
                            source,
                            invocation,
                            row,
                            &operation.origin,
                            &cancel(),
                        )?;
                        let CreationPhase::Invocation(row) = &mut operation.phase else {
                            unreachable!()
                        };
                        row.request.cleanup = Some(cleanup);
                        Ok(OperationResult::Retain(cleanup))
                    })
                    .unwrap(),
            )
        } else {
            None
        };
        if narrow {
            f.dom.settle_hosted_history_completion(invocation, history);
            assert!(phase(&f, invocation).history == HistoryDependency::Settled(history));
            assert_eq!(phase(&f, invocation).request.cleanup, cleanup);
        }
        if complete {
            f.dom
                .with_hosted_operation(invocation, |_, operation| {
                    assert_eq!(operation.request().key.operation, invocation);
                    Ok(OperationResult::Complete(()))
                })
                .unwrap();
            f.source
                .retire(
                    Role::Operation,
                    invocation.source,
                    invocation.slot,
                    invocation.generation,
                    &cancel(),
                )
                .unwrap();
        } else {
            f.retire_rows(&[invocation]);
        }
        f.retire_rows(&[history]);
        if let Some(cleanup) = cleanup {
            f.retire_rows(&[cleanup]);
        }
        drop(work);
        let settled = (
            f.source.retained_storage_bytes(),
            f.dom.hosted_operations.inspect_storage_bytes(),
        );
        if let Some(expected) = baseline {
            assert_eq!(
                settled, expected,
                "fixed issuer slots after cycle {cycle}; permanent retirement must release generation-qualified index suffixes"
            );
        } else {
            baseline = Some(settled);
        }
    }
}
#[test]
fn permanent_controller_remove_reclaims_retired_generation_paths_at_fixed_slot_extent() {
    retained_controller_cycles(false, false);
}
#[test]
fn permanent_controller_finish_reclaims_retired_generation_paths_at_fixed_slot_extent() {
    retained_controller_cycles(true, false);
}
#[test]
fn permanently_removed_narrowed_projections_release_the_original_allowed_paths() {
    retained_controller_cycles(false, true);
}

#[test]
fn focused_designation_hosted_frame_and_iframe_are_fresh_and_survive_service_expiry() {
    for local in ["frame", "iframe"] {
        let mut fixture = Fixture::new();
        let (_, child) = insert_absent_src_element(&mut fixture, local);
        let captured = fixture.dom.snapshot();
        for document in [fixture.creation.document, child.document] {
            assert_eq!(
                captured
                    .document(document)
                    .unwrap()
                    .focused_area_designation(),
                crate::internal::document::DomFocusedAreaDesignation::KnownNull
            );
        }
        drop(fixture.source);
        for document in [fixture.creation.document, child.document] {
            assert_eq!(
                fixture
                    .dom
                    .snapshot()
                    .document(document)
                    .unwrap()
                    .focused_area_designation(),
                crate::internal::document::DomFocusedAreaDesignation::KnownNull
            );
            assert_eq!(
                captured
                    .document(document)
                    .unwrap()
                    .focused_area_designation(),
                crate::internal::document::DomFocusedAreaDesignation::KnownNull
            );
        }
    }
}

fn native_child_request(
    lease: &mut source::operation::DomSourceLease<'_>,
) -> DomHostedChildRequest {
    match lease.advance(&cancel()).unwrap() {
        DomSourceProgress::Request(DomSourceRequest::HostedChild(request)) => Some(request),
        _ => None,
    }
    .expect("original native child lifecycle request")
}

fn drive_native_insertion<const N: usize>(
    f: &mut Fixture,
    container: NodeId,
    parent: NodeId,
    before: Option<NodeId>,
) -> [DomHostedCreation; N] {
    let mut lease = f
        .dom
        .begin_source_edit(
            crate::internal::source::DomSourceEdit::insert_before(container, parent, before),
            source_limits(),
            &cancel(),
        )
        .unwrap();
    let children = std::array::from_fn(|_| {
        let request = native_child_request(&mut lease);
        drive_absent_src_child(&mut lease, &mut f.source, &request)
    });
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    children
}

fn drive_native_removal(f: &mut Fixture, container: NodeId) {
    let mut lease = f
        .dom
        .begin_source_edit(
            crate::internal::source::DomSourceEdit::detach(container),
            source_limits(),
            &cancel(),
        )
        .unwrap();
    let request = native_child_request(&mut lease);
    lease
        .prepare_hosted_child_removal(&mut f.source, &request, &cancel())
        .unwrap();
    let packet = match lease.hosted_child_step(&request).unwrap() {
        DomHostedChildStep::Removal(packet) => Some(packet),
        _ => None,
    }
    .expect("original native child removal packet");
    let mut publication = Some(f.source.seal_child_removal(packet, &cancel()).unwrap());
    lease
        .accept_hosted_child_removal(&mut f.source, &request, &mut publication, &cancel())
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}

fn complete_native_structure(f: &mut Fixture, edit: crate::internal::source::DomSourceEdit) {
    let mut lease = f
        .dom
        .begin_source_edit(edit, source_limits(), &cancel())
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}

#[test]
fn fragment_child_navigable_links_have_distinct_insertion_groups() {
    use surgeist_html::values::Namespace;

    let mut f = Fixture::new();
    let document = f.creation.document;
    let parent = f
        .dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(document, None);
    let fragment = batch.create_fragment(&closed);
    let frame = batch.create_element_ns(&closed, Namespace::html(), "frame");
    let iframe = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    batch.insert_before(&frame, &fragment, None);
    batch.insert_before(&iframe, &fragment, None);
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let [fragment, frame, iframe] =
        [&fragment, &frame, &iframe].map(|ticket| commit.created(ticket).unwrap().unwrap());
    assert_eq!(
        f.dom.snapshot().node(fragment).unwrap().children(),
        &[frame, iframe]
    );

    let children = drive_native_insertion::<2>(&mut f, fragment, parent, None);
    let snapshot = f.dom.snapshot();
    assert!(snapshot.node(fragment).unwrap().children().is_empty());
    let [first, second] = std::array::from_fn(|index| {
        let container = [frame, iframe][index];
        let navigable = snapshot
            .hosted_navigable(children[index].navigable)
            .unwrap();
        assert_eq!(snapshot.node(container).unwrap().parent(), Some(parent));
        assert_eq!(navigable.container(), Some(container));
        let occurrence = navigable.container_insertion().unwrap();
        assert_eq!(occurrence.document(), document);
        assert_eq!(occurrence.preorder(), 0);
        occurrence
    });
    assert!(first.group() < second.group());
}

#[test]
fn child_insertion_provenance_follows_structural_groups_across_moves_and_reinsertion() {
    use crate::internal::shadow::DomShadowRootInit;
    use crate::internal::source::DomSourceEdit;
    use surgeist_html::{construction::HtmlShadowRootMode, values::Namespace};

    let mut f = Fixture::new();
    let (old, old_child) = insert_absent_src_frame(&mut f);
    let document = f.creation.document;
    let snapshot = f.dom.snapshot();
    let parent = snapshot.node(old).unwrap().parent().unwrap();
    let prior = snapshot
        .hosted_navigable(old_child.navigable)
        .unwrap()
        .container_insertion()
        .unwrap();

    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(document, None);
    let subtree = batch.create_element_ns(&closed, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&subtree, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let light = batch.create_element_ns(&closed, Namespace::html(), "frame");
    let hidden = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    batch.insert_before(&light, &subtree, None);
    batch.insert_before(&hidden, &shadow, None);
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let shadow = commit.attached_shadow_root(&shadow).unwrap().unwrap();
    let [subtree, light, hidden] =
        [&subtree, &light, &hidden].map(|ticket| commit.created(ticket).unwrap().unwrap());

    let before = f.dom.snapshot();
    let mut refused = DomBatch::new();
    refused.insert_before(subtree, parent, Some(old.into()));
    assert_eq!(
        f.dom.apply(refused, &cancel()).unwrap_err(),
        DomError::HostedSourceUnavailable
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.snapshot().node(subtree).unwrap().parent(), None);

    let [hidden_child, light_child] = drive_native_insertion(&mut f, subtree, parent, Some(old));
    let snapshot = f.dom.snapshot();
    let insertion = |snapshot: &DomSnapshot, child: DomHostedCreation| {
        snapshot
            .hosted_navigable(child.navigable)
            .unwrap()
            .container_insertion()
            .unwrap()
    };
    let hidden_insert = insertion(&snapshot, hidden_child);
    let light_insert = insertion(&snapshot, light_child);
    assert_eq!(hidden_insert.document(), document);
    assert_eq!(light_insert.document(), document);
    assert_eq!(hidden_insert.group(), prior.group() + 1);
    assert_eq!(hidden_insert.group(), light_insert.group());
    assert!(hidden_insert.preorder() < light_insert.preorder());
    let current_children = |dom: &Dom, document| {
        let mut found = Vec::new();
        let count = dom
            .visit_document_tree_child_navigables(
                document,
                100_000,
                || Ok::<_, DomHostedError>(()),
                &cancel(),
                |child| {
                    found.push((
                        child.container().unwrap(),
                        child.container_insertion().unwrap(),
                    ));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(count, found.len());
        found
    };
    assert_eq!(
        current_children(&f.dom, document),
        [(light, light_insert), (old, prior)]
    );

    complete_native_structure(
        &mut f,
        DomSourceEdit::move_before(old, parent, Some(subtree)),
    );
    assert_eq!(
        current_children(&f.dom, document),
        [(old, prior), (light, light_insert)]
    );

    complete_native_structure(
        &mut f,
        DomSourceEdit::move_before(light, shadow, Some(hidden)),
    );
    assert_eq!(current_children(&f.dom, document), [(old, prior)]);
    assert_eq!(insertion(&f.dom.snapshot(), light_child), light_insert);

    complete_native_structure(
        &mut f,
        DomSourceEdit::move_before(hidden, parent, Some(old)),
    );
    assert_eq!(
        current_children(&f.dom, document),
        [(hidden, hidden_insert), (old, prior)]
    );

    drive_native_removal(&mut f, hidden);
    assert_eq!(current_children(&f.dom, document), [(old, prior)]);
    assert_eq!(insertion(&f.dom.snapshot(), hidden_child), hidden_insert);

    let [same_document_child] = drive_native_insertion(&mut f, hidden, parent, None);
    let same_document = insertion(&f.dom.snapshot(), same_document_child);
    assert_ne!(same_document_child.navigable, hidden_child.navigable);
    assert_eq!(same_document.document(), document);
    assert!(same_document.group() > hidden_insert.group());
    assert_eq!(
        current_children(&f.dom, document),
        [(old, prior), (hidden, same_document)]
    );
    drive_native_removal(&mut f, hidden);

    let destination = old_child.document;
    let snapshot = f.dom.snapshot();
    let destination_parent = snapshot
        .document(destination)
        .unwrap()
        .document_element()
        .unwrap();
    complete_native_structure(&mut f, DomSourceEdit::adopt(hidden, destination.node()));

    let [reinserted] = drive_native_insertion(&mut f, hidden, destination_parent, None);
    assert_ne!(reinserted.navigable, hidden_child.navigable);
    let current = insertion(&f.dom.snapshot(), reinserted);
    assert_eq!(current.document(), destination);
    assert!(current.group() > same_document.group());
    assert_eq!(current_children(&f.dom, destination), [(hidden, current)]);
    assert_eq!(current_children(&f.dom, document), [(old, prior)]);

    let stale = DomHostedNavigableId {
        generation: reinserted.navigable.generation + 1,
        ..reinserted.navigable
    };
    assert!(matches!(
        f.dom.hosted_navigable(stale),
        Err(DomHostedError::StaleIdentity)
    ));
}

fn named_value(dom: &Dom, global: DomHostedGlobalId, name: DocumentString) -> DomWindowNamedValue {
    dom.window_named_value(
        global,
        name,
        1024,
        100_000,
        || Ok::<_, DomHostedError>(()),
        &cancel(),
    )
    .unwrap()
}

#[test]
fn window_named_value_selects_the_first_child_before_origin_and_preserves_its_native_name() {
    use crate::internal::source::DomSourceEdit;
    use surgeist_html::values::Namespace;

    let mut f = Fixture::new();
    let document = f.creation.document;
    let global = f.creation.global;
    let target = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let nearby = DocumentString::from_utf16(vec![0xd800, 0, 0xdc01]);
    let parent = f
        .dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(document, None);
    let isolated = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    let same = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    batch.set_attribute(&isolated, "sandbox", "");
    batch.set_attribute(&isolated, "name", target.clone());
    batch.set_attribute(&same, "name", target.clone());
    let contributor = batch.create_element_ns(document, Namespace::html(), "div");
    let extra = batch.create_element_ns(document, Namespace::html(), "div");
    batch.insert_before(&contributor, parent, None);
    batch.insert_before(&extra, parent, None);
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let [isolated, same, contributor, extra] = [&isolated, &same, &contributor, &extra]
        .map(|ticket| commit.created(ticket).unwrap().unwrap());
    let [opaque] = drive_native_insertion(&mut f, isolated, parent, None);
    let [inherited] = drive_native_insertion(&mut f, same, parent, None);
    let snapshot = f.dom.snapshot();
    let origin = snapshot.document(document).unwrap().metadata().origin();
    assert_ne!(
        snapshot
            .document(opaque.document)
            .unwrap()
            .metadata()
            .origin(),
        origin
    );
    assert_eq!(
        snapshot
            .document(inherited.document)
            .unwrap()
            .metadata()
            .origin(),
        origin
    );
    for child in [opaque, inherited] {
        let navigable = snapshot.hosted_navigable(child.navigable).unwrap();
        assert_eq!(
            snapshot
                .hosted_entry(navigable.active_entry())
                .unwrap()
                .target_name(),
            Some(&target)
        );
    }
    assert!(matches!(
        named_value(&f.dom, global, target.clone()),
        DomWindowNamedValue::Unsupported
    ));
    assert!(matches!(
        named_value(&f.dom, global, nearby),
        DomWindowNamedValue::Unsupported
    ));
    assert!(matches!(
        named_value(&f.dom, global, "".into()),
        DomWindowNamedValue::Unsupported
    ));

    let mut edit = DomBatch::new();
    edit.set_attribute(contributor, "name", target.clone());
    f.dom.apply(edit, &cancel()).unwrap();
    assert!(matches!(
        named_value(&f.dom, global, target.clone()),
        DomWindowNamedValue::Unsupported
    ));

    let mut edit = DomBatch::new();
    edit.set_attribute(contributor, "id", target.clone());
    edit.set_attribute(extra, "id", target.clone());
    edit.set_attribute(isolated, "name", "authored-new");
    edit.set_attribute(same, "name", "authored-new");
    f.dom.apply(edit, &cancel()).unwrap();
    assert!(
        matches!(named_value(&f.dom, global, target.clone()), DomWindowNamedValue::Child(context) if context == opaque.context)
    );
    assert!(matches!(
        named_value(&f.dom, global, "authored-new".into()),
        DomWindowNamedValue::Unsupported
    ));

    let child_work = |dom: &Dom, expected| {
        let steps = std::cell::Cell::new(0);
        let outcome = dom
            .window_named_value(
                global,
                target.clone(),
                1024,
                100_000,
                || {
                    steps.set(steps.get() + 1);
                    Ok::<_, DomHostedError>(())
                },
                &cancel(),
            )
            .unwrap();
        assert!(matches!(outcome, DomWindowNamedValue::Child(context) if context == expected));
        steps.get()
    };
    let cross_origin_work = child_work(&f.dom, opaque.context);
    let mut edit = DomBatch::new();
    // The first Element already supplies the support the first child needs.
    edit.set_attribute(extra, "id", "later-element");
    f.dom.apply(edit, &cancel()).unwrap();
    assert_eq!(child_work(&f.dom, opaque.context), cross_origin_work);

    complete_native_structure(
        &mut f,
        DomSourceEdit::move_before(same, parent, Some(isolated)),
    );
    assert!(
        matches!(named_value(&f.dom, global, target.clone()), DomWindowNamedValue::Child(context) if context == inherited.context)
    );
    let same_origin_work = child_work(&f.dom, inherited.context);
    let mut edit = DomBatch::new();
    edit.remove_attribute(contributor, "id");
    edit.remove_attribute(extra, "id");
    f.dom.apply(edit, &cancel()).unwrap();
    assert!(
        matches!(named_value(&f.dom, global, target.clone()), DomWindowNamedValue::Child(context) if context == inherited.context)
    );

    assert_eq!(child_work(&f.dom, inherited.context), same_origin_work);

    drive_native_removal(&mut f, same);
    assert!(matches!(
        named_value(&f.dom, global, target.clone()),
        DomWindowNamedValue::Unsupported
    ));
    assert!(matches!(
        named_value(&f.dom, opaque.global, "".into()),
        DomWindowNamedValue::Unsupported
    ));
    drive_native_removal(&mut f, isolated);
    let retiring = f.dom.snapshot();
    let navigable = retiring.hosted_navigable(opaque.navigable).unwrap();
    assert!(navigable.is_retiring());
    assert!(!navigable.is_retired());
    assert_eq!(navigable.active_document(), Some(opaque.document));
    assert_eq!(retiring.global_document(opaque.global), Ok(opaque.document));
    let resolve_retiring = |name| {
        f.dom.window_named_value(
            opaque.global,
            name,
            1024,
            100_000,
            || Ok::<_, DomHostedError>(()),
            &cancel(),
        )
    };
    assert!(matches!(
        resolve_retiring(target.clone()),
        Err(DomHostedError::StaleIdentity)
    ));
    let empty = resolve_retiring("".into());
    assert!(
        matches!(&empty, Err(DomHostedError::StaleIdentity)),
        "retiring Window with an empty name: {empty:?}",
    );

    let mut edit = DomBatch::new();
    edit.set_attribute(contributor, "id", target.clone());
    f.dom.apply(edit, &cancel()).unwrap();
    assert!(
        matches!(named_value(&f.dom, global, target.clone()), DomWindowNamedValue::Element(node) if node == contributor)
    );
    let mut edit = DomBatch::new();
    edit.set_attribute(extra, "id", target.clone());
    f.dom.apply(edit, &cancel()).unwrap();
    let collection = match named_value(&f.dom, global, target) {
        DomWindowNamedValue::Collection(value) => Some(value),
        _ => None,
    }
    .expect("two actual Element contributors");
    let cancelled = cancel();
    let read = f
        .dom
        .read_html_collection(
            &collection,
            crate::tree::DomHtmlCollectionLimits::new(100_000, 8, 512),
            || Ok::<_, DomError>(()),
            &cancelled,
        )
        .unwrap();
    assert_eq!(read.item(0).unwrap(), Some(contributor));
    assert_eq!(read.item(1).unwrap(), Some(extra));
    assert_eq!(read.length().unwrap(), 2);
}

#[derive(Debug, PartialEq, Eq)]
enum WindowReadError {
    Native(DomHostedError),
    Refused,
}
impl From<DomHostedError> for WindowReadError {
    fn from(error: DomHostedError) -> Self {
        Self::Native(error)
    }
}

#[test]
fn window_named_value_revalidates_the_live_global_after_element_admission() {
    use std::{
        cell::{Cell, RefCell},
        sync::atomic::Ordering,
    };

    let Fixture {
        mut dom,
        source,
        creation,
    } = Fixture::new();
    let target = "native-only";
    let parent = dom
        .snapshot()
        .document(creation.document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut edit = DomBatch::new();
    let contributor = edit.create_element(creation.document, "div");
    edit.set_attribute(&contributor, "id", target);
    edit.insert_before(&contributor, parent, None);
    dom.apply(edit, &cancel()).unwrap();

    let resolve = |global, work| {
        dom.window_named_value(
            global,
            target.into(),
            1024,
            work,
            || Ok::<_, DomHostedError>(()),
            &cancel(),
        )
    };
    let mut stale = creation.global;
    stale.generation += 1;
    assert!(matches!(
        resolve(stale, 100_000),
        Err(DomHostedError::StaleIdentity)
    ));
    let foreign = Fixture::new();
    assert!(matches!(
        resolve(foreign.creation.global, 100_000),
        Err(DomHostedError::ForeignSource)
    ));

    let steps = Cell::new(0);
    assert!(matches!(
        dom.window_named_value(
            creation.global,
            target.into(),
            1024,
            100_000,
            || {
                steps.set(steps.get() + 1);
                Ok::<_, DomHostedError>(())
            },
            &cancel(),
        ),
        Ok(DomWindowNamedValue::Element(_))
    ));
    let exact = steps.get();
    assert!(matches!(
        resolve(creation.global, exact),
        Ok(DomWindowNamedValue::Element(_))
    ));
    assert!(
        matches!(resolve(creation.global, exact - 1), Err(DomHostedError::WorkLimit { limit }) if limit == exact - 1)
    );

    steps.set(0);
    let refusal = dom.window_named_value(
        creation.global,
        target.into(),
        1024,
        exact,
        || {
            steps.set(steps.get() + 1);
            if steps.get() == exact {
                Err(WindowReadError::Refused)
            } else {
                Ok(())
            }
        },
        &cancel(),
    );
    assert_eq!(refusal.unwrap_err(), WindowReadError::Refused);

    let cancelled = cancel();
    steps.set(0);
    let result = dom.window_named_value(
        creation.global,
        target.into(),
        1024,
        exact,
        || {
            steps.set(steps.get() + 1);
            if steps.get() == exact {
                cancelled.store(true, Ordering::Relaxed);
            }
            Ok::<_, DomHostedError>(())
        },
        &cancelled,
    );
    assert!(matches!(
        result,
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));

    let source = RefCell::new(Some(source));
    steps.set(0);
    let result = dom.window_named_value(
        creation.global,
        target.into(),
        1024,
        exact,
        || {
            steps.set(steps.get() + 1);
            if steps.get() == exact {
                drop(source.borrow_mut().take());
            }
            Ok::<_, DomHostedError>(())
        },
        &cancel(),
    );
    assert!(matches!(result, Err(DomHostedError::SourceExpired)));
}

#[test]
fn window_named_value_charges_one_exact_dispatch_scope_across_child_and_element_matching() {
    use crate::internal::{dispatch::DomDispatchCustodyLimits, source::DomSourceEdit};
    use std::cell::Cell;
    use surgeist_html::values::Namespace;

    let mut f = Fixture::new();
    let custody =
        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap();
    f.dom
        .set_limits(f.dom.limits().with_dispatch_custody(custody))
        .unwrap();
    let mut edit = DomBatch::new();
    let actor = edit.create_html_document(f.creation.document, None);
    let commit = f.dom.apply(edit, &cancel()).unwrap();
    let actor = commit.document(&actor).unwrap();
    let actor_parent = f
        .dom
        .snapshot()
        .document(actor)
        .unwrap()
        .document_element()
        .unwrap();
    let parent = f
        .dom
        .snapshot()
        .document(f.creation.document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut edit = DomBatch::new();
    let iframe = edit.create_element_ns(actor, Namespace::html(), "iframe");
    edit.set_attribute(&iframe, "sandbox", "");
    edit.set_attribute(&iframe, "name", "marker");
    let dialog = edit.create_element_ns(actor, Namespace::html(), "dialog");
    let target = edit.create_element_ns(f.creation.document, Namespace::html(), "div");
    edit.set_attribute(&target, "id", "marker");
    edit.insert_before(&target, parent, None);
    edit.insert_before(&dialog, actor_parent, None);
    let commit = f.dom.apply(edit, &cancel()).unwrap();
    let iframe = commit.created(&iframe).unwrap().unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let [opaque] = drive_native_insertion(&mut f, iframe, parent, None);
    let executor = f.dom.bind_source_event_executor(&cancel()).unwrap();
    let event = {
        let mut edit = f
            .dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(dialog),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        match edit.advance(&cancel()).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Event(event)) => Some(event),
            _ => None,
        }
        .expect("actual native dialog event")
    };
    let admissions = Cell::new(0);
    let global = f.creation.global;
    let resolve = |dom: &Dom| {
        dom.window_named_value(
            global,
            "marker".into(),
            1024,
            100_000,
            || {
                admissions.set(admissions.get() + 1);
                Ok::<_, DomHostedError>(())
            },
            &cancel(),
        )
    };
    assert!(matches!(
        resolve(&f.dom),
        Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
    ));
    let permit = executor.begin_dispatch(&f.dom, &event, &cancel()).unwrap();
    let scope = executor.enter_dispatch(&f.dom, &permit, &cancel()).unwrap();
    let before = f.dom.pending_source_edit().unwrap().spent_work();
    assert!(matches!(
        resolve(&f.dom), Ok(DomWindowNamedValue::Child(actual)) if actual == opaque.context
    ));
    let after = f.dom.pending_source_edit().unwrap().spent_work();
    assert_eq!(after - before, admissions.get() + 1);
    let required = after - before;
    let stamp = f.dom.snapshot().stamp();
    for shortfall in [0, 1] {
        let allowance = required - shortfall;
        let mut limits = source_limits();
        let spent = f.dom.pending_source_edit().unwrap().spent_work();
        // Leave room for the real resume checks, then spend the difference
        // through the original event to reach the exact remaining allowance.
        limits.work = spent + allowance + 4096;
        drop(
            f.dom
                .resume_source_edit(event.operation(), limits, &cancel())
                .unwrap(),
        );
        let remaining = limits.work - f.dom.pending_source_edit().unwrap().spent_work();
        assert!(remaining >= allowance);
        for _ in 0..remaining - allowance {
            f.dom.source_event_step(&event, &cancel()).unwrap();
        }

        admissions.set(0);
        let result = resolve(&f.dom);
        if shortfall == 0 {
            assert!(
                matches!(result, Ok(DomWindowNamedValue::Child(actual)) if actual == opaque.context)
            );
        } else {
            assert_eq!(
                result.unwrap_err(),
                DomHostedError::Dom(DomError::SourceWorkLimit { limit: limits.work }),
            );
        }
        let current = f.dom.pending_source_edit().unwrap();
        assert_eq!(current.operation(), event.operation());
        assert_eq!(current.spent_work(), limits.work);
        assert_eq!(admissions.get() + 1, required);
        assert_eq!(f.dom.snapshot().stamp(), stamp);
    }

    drop(executor);
    assert!(matches!(
        resolve(&f.dom),
        Err(DomHostedError::Dom(DomError::InvalidSourceEvent))
    ));
    drop(scope);
}

mod window_indexed;
