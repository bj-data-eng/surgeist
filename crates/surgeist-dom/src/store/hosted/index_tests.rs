//! Identity-index admission only. Native Documents are genuine payload IDs;
//! these fixtures do not certify hosted Documents, agents, loops or services.
use super::*;
use std::mem::size_of;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

type IndexEntry = Option<(DomHostedGlobalId, DocumentId)>;

fn cancellation() -> AtomicBool {
    AtomicBool::new(false)
}

struct Fixture {
    dom: Dom,
    source: DomHostedSource,
    host: DomHostedHostId,
    documents: [DocumentId; 2],
}

impl Fixture {
    fn new() -> Self {
        let cancel = cancellation();
        let mut dom = Dom::new(DomLimits::new(64, 4096, 64, 0, 0, 16, 4_000_000).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let first = batch.create_document();
        let second = batch.create_document();
        let commit = dom.apply(batch, &cancel).unwrap();
        let documents = [
            commit.document(&first).unwrap(),
            commit.document(&second).unwrap(),
        ];
        let mut source = dom
            .register_hosted_source(
                DomHostedLimits::new(
                    DomHostedRecordLimits::new(64, 64, 64, 64, 64, 64, 64),
                    8_000_000,
                    8_000_000,
                    1_000_000,
                ),
                &cancel,
            )
            .unwrap();
        let host = source.issue_host(&cancel).unwrap();
        Self {
            dom,
            source,
            host,
            documents,
        }
    }

    fn global(&mut self) -> (DomHostedGlobalId, Arc<Work>) {
        let cancel = cancellation();
        let prefix = self.source.prepare_top_level(self.host, &cancel).unwrap();
        let work = prefix.work;
        let operation = self.source.begin_operation(&work, 10, 0, &cancel).unwrap();
        let mut request = Request {
            key: RequestKey {
                operation,
                phase: 0,
            },
            host: prefix.host,
            context: prefix.context,
            work: work.clone(),
            parent: None,
        };
        advance(&self.source, &mut request);
        let origin = self
            .dom
            .snapshot()
            .document(self.documents[0])
            .unwrap()
            .metadata()
            .origin()
            .clone();
        let inputs =
            crate::document::HostedDocumentInputs::form(&self.source, &work, &origin, &cancel)
                .unwrap();
        let execution = DomHostedExecutionRequest {
            request,
            origin,
            inputs,
        };
        let global = self.source.issue_global(&execution, &cancel).unwrap();
        let mut request = execution.request;
        // Exercise the original issuer's exact role/phase predicates. These
        // permits do not publish a canonical association or acknowledge services.
        for _ in 0..3 {
            advance(&self.source, &mut request);
        }
        let entry = DomHostedEntryRequest {
            request,
            document: self.documents[0],
            global,
        };
        self.source.issue_entry(&entry, &cancel).unwrap();
        self.source.issue_navigable(&entry, &cancel).unwrap();
        let mut request = entry.request;
        advance(&self.source, &mut request);
        self.source
            .permit(&request, self.source.revision().unwrap(), true, &cancel)
            .unwrap()
            .commit();
        (global, work)
    }

    fn used(&self) -> usize {
        self.source.limits().retained_bytes() - self.source.remaining_bytes().unwrap()
    }

    fn limits(&mut self, retained: usize, scratch: usize, work: u64) {
        let records = self.source.limits().records();
        self.source
            .set_limits(
                DomHostedLimits::new(records, retained, scratch, work),
                &cancellation(),
            )
            .unwrap();
    }
}

fn advance(source: &DomHostedSource, request: &mut Request) {
    source
        .permit(request, source.revision().unwrap(), false, &cancellation())
        .unwrap()
        .commit();
    request.key.phase += 1;
}

fn establish(
    fixture: &Fixture,
    old: Option<&GlobalDocumentIndex>,
    global: DomHostedGlobalId,
    document: DocumentId,
    work: &Work,
    cancel: &AtomicBool,
) -> Result<Arc<GlobalDocumentIndex>, DomHostedError> {
    GlobalDocumentIndex::establish(old, global, document, &fixture.source, work, None, cancel)
}

#[test]
fn global_index_replacement_admits_old_and_new_actual_backing_at_the_same_time() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (first, work) = fixture.global();
    let old = establish(&fixture, None, first, fixture.documents[0], &work, &cancel).unwrap();
    let captured_old = old.clone();
    let old_bytes =
        size_of::<GlobalDocumentIndex>() + old.entries.capacity() * size_of::<IndexEntry>();
    assert_eq!(old.retained_payload_bytes().unwrap(), old_bytes);
    drop(work);
    let (next, work) = fixture.global();
    assert_eq!(next.slot, first.slot + 1);
    let baseline = fixture.used();
    let old_limits = fixture.source.limits();
    let required = old.entries.len().max(next.slot + 1);
    let requested = size_of::<GlobalDocumentIndex>() + required * size_of::<IndexEntry>();
    let one_short = baseline + requested - 1;
    fixture.limits(one_short, old_limits.scratch_bytes(), old_limits.work());
    assert!(
        matches!(establish(&fixture, Some(&old), next, fixture.documents[1], &work, &cancel),
        Err(DomHostedError::Limit { resource: DomHostedResource::RetainedBytes, limit }) if limit == one_short)
    );
    assert_eq!(fixture.used(), baseline);
    assert_eq!(old.document(first), Ok(fixture.documents[0]));
    assert_eq!(old.document(next), Err(DomHostedError::StaleIdentity));

    fixture.limits(
        old_limits.retained_bytes(),
        old_limits.scratch_bytes(),
        old_limits.work(),
    );
    let candidate = establish(
        &fixture,
        Some(&old),
        next,
        fixture.documents[1],
        &work,
        &cancel,
    )
    .unwrap();
    let actual =
        size_of::<GlobalDocumentIndex>() + candidate.entries.capacity() * size_of::<IndexEntry>();
    assert_eq!(candidate.retained_payload_bytes().unwrap(), actual);
    assert_eq!(fixture.used(), baseline + actual);
    let peak = baseline + actual;
    assert_eq!(
        fixture.source.set_limits(
            DomHostedLimits::new(
                old_limits.records(),
                peak - 1,
                old_limits.scratch_bytes(),
                old_limits.work()
            ),
            &cancel
        ),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            limit: peak - 1
        })
    );
    fixture.limits(peak, old_limits.scratch_bytes(), old_limits.work());
    assert_eq!(fixture.source.remaining_bytes().unwrap(), 0);
    assert_eq!(candidate.document(first), Ok(fixture.documents[0]));
    assert_eq!(candidate.document(next), Ok(fixture.documents[1]));
    drop(candidate);
    assert_eq!(fixture.used(), baseline);
    // The same admitted representation can be formed at its exact measured
    // peak. Extra allocator capacity is always assessed as actual backing.
    let exact = establish(
        &fixture,
        Some(&old),
        next,
        fixture.documents[1],
        &work,
        &cancel,
    )
    .unwrap();
    assert_eq!(
        fixture.used(),
        baseline + exact.retained_payload_bytes().unwrap()
    );
    assert_eq!(
        captured_old.document(next),
        Err(DomHostedError::StaleIdentity)
    );
    drop(old);
    assert_eq!(fixture.used(), baseline + actual);
    drop(captured_old);
    assert_eq!(fixture.used(), baseline + actual - old_bytes);
}

#[test]
fn global_index_scratch_prices_capacity_and_header_not_only_present_mappings() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    for _ in 0..2 {
        let (_, work) = fixture.global();
        drop(work);
    }
    let (global, work) = fixture.global();
    assert_eq!(global.slot, 2);
    let baseline = fixture.used();
    let old_limits = fixture.source.limits();
    let roomy = establish(&fixture, None, global, fixture.documents[1], &work, &cancel).unwrap();
    assert_eq!(roomy.entries.len(), 3);
    assert_eq!(
        roomy.entries.iter().filter(|entry| entry.is_some()).count(),
        1
    );
    let actual_backing = roomy.entries.capacity() * size_of::<IndexEntry>();
    let scratch = size_of::<Vec<IndexEntry>>() + actual_backing;
    assert_eq!(
        roomy.retained_payload_bytes().unwrap(),
        size_of::<GlobalDocumentIndex>() + actual_backing
    );
    drop(roomy);
    assert_eq!(fixture.used(), baseline);
    fixture.limits(old_limits.retained_bytes(), scratch - 1, old_limits.work());
    let spent = work.spent.load(Ordering::Relaxed);
    assert!(
        matches!(establish(&fixture, None, global, fixture.documents[1], &work, &cancel),
        Err(DomHostedError::Limit { resource: DomHostedResource::ScratchBytes, limit }) if limit == scratch - 1)
    );
    assert_eq!(fixture.used(), baseline);
    assert_eq!(work.spent.load(Ordering::Relaxed), spent + 1);
    fixture.limits(old_limits.retained_bytes(), scratch, old_limits.work());
    let exact = establish(&fixture, None, global, fixture.documents[1], &work, &cancel).unwrap();
    assert_eq!(exact.document(global), Ok(fixture.documents[1]));
    assert_eq!(
        size_of::<Vec<IndexEntry>>() + exact.entries.capacity() * size_of::<IndexEntry>(),
        scratch
    );
    drop(exact);
    assert_eq!(fixture.used(), baseline);
}

#[test]
fn global_index_late_work_stop_releases_candidate_without_refunding_original_work() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (first, work) = fixture.global();
    let old = establish(&fixture, None, first, fixture.documents[0], &work, &cancel).unwrap();
    drop(work);
    let (next, work) = fixture.global();
    let baseline = fixture.used();
    let revision = fixture.source.revision().unwrap();
    let old_limits = fixture.source.limits();
    let spent = work.spent.load(Ordering::Relaxed);
    fixture.limits(
        old_limits.retained_bytes(),
        old_limits.scratch_bytes(),
        spent + 1,
    );
    // The first step passes; allocation and Source retention occur before the
    // next admitted step, which is the first actual old-entry copy.
    assert!(
        matches!(establish(&fixture, Some(&old), next, fixture.documents[1], &work, &cancel),
        Err(DomHostedError::WorkLimit { limit }) if limit == spent + 1)
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), spent + 1);
    assert_eq!(fixture.used(), baseline);
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(old.document(first), Ok(fixture.documents[0]));
    assert_eq!(old.document(next), Err(DomHostedError::StaleIdentity));
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        establish(
            &fixture,
            Some(&old),
            next,
            fixture.documents[1],
            &work,
            &cancel
        ),
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));
    assert_eq!(work.spent.load(Ordering::Relaxed), spent + 1);
    assert_eq!(fixture.used(), baseline);
    cancel.store(false, Ordering::Relaxed);
    assert!(
        matches!(establish(&fixture, Some(&old), next, fixture.documents[1], &work, &cancel),
        Err(DomHostedError::WorkLimit { limit }) if limit == spent + 1)
    );
    let required = old.entries.len().max(next.slot + 1);
    // One initial step, one per copied/filled initialized entry, and one write.
    let complete_work = u64::try_from(required + 2).unwrap();
    fixture.limits(
        old_limits.retained_bytes(),
        old_limits.scratch_bytes(),
        spent + 1 + complete_work,
    );
    let recovered = establish(
        &fixture,
        Some(&old),
        next,
        fixture.documents[1],
        &work,
        &cancel,
    )
    .unwrap();
    assert_eq!(
        work.spent.load(Ordering::Relaxed),
        spent + 1 + complete_work
    );
    assert_eq!(recovered.document(first), Ok(fixture.documents[0]));
    assert_eq!(recovered.document(next), Ok(fixture.documents[1]));
    drop(recovered);
    assert_eq!(fixture.used(), baseline);
}

#[test]
fn global_index_duplicate_binding_keeps_old_payload_and_reclaims_failed_candidate() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (global, work) = fixture.global();
    let old = establish(&fixture, None, global, fixture.documents[0], &work, &cancel).unwrap();
    let baseline = fixture.used();
    let spent = work.spent.load(Ordering::Relaxed);
    assert!(matches!(
        establish(
            &fixture,
            Some(&old),
            global,
            fixture.documents[1],
            &work,
            &cancel
        ),
        Err(DomHostedError::StaleBasis)
    ));
    assert_eq!(fixture.used(), baseline);
    assert_eq!(
        work.spent.load(Ordering::Relaxed),
        spent + u64::try_from(old.entries.len() + 2).unwrap()
    );
    assert_eq!(old.document(global), Ok(fixture.documents[0]));
}

#[test]
fn global_index_historical_capture_survives_real_role_slot_generation_reuse() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (prior, work) = fixture.global();
    let old = establish(&fixture, None, prior, fixture.documents[0], &work, &cancel).unwrap();
    let captured = old.clone();
    drop(work);
    fixture
        .source
        .retire(
            Role::Global,
            prior.source,
            prior.slot,
            prior.generation,
            &cancel,
        )
        .unwrap();
    let (current, work) = fixture.global();
    assert_eq!(current.slot, prior.slot);
    assert_eq!(current.generation, prior.generation + 1);
    let new = establish(
        &fixture,
        Some(&old),
        current,
        fixture.documents[1],
        &work,
        &cancel,
    )
    .unwrap();
    assert_eq!(captured.document(prior), Ok(fixture.documents[0]));
    assert_eq!(
        captured.document(current),
        Err(DomHostedError::StaleIdentity)
    );
    assert_eq!(new.document(prior), Err(DomHostedError::StaleIdentity));
    assert_eq!(new.document(current), Ok(fixture.documents[1]));
    drop(fixture.source);
    // Pure immutable identity history remains readable after live-use permission
    // expires; this does not make either native payload a hosted Document.
    assert_eq!(captured.document(prior), Ok(fixture.documents[0]));
    assert_eq!(new.document(current), Ok(fixture.documents[1]));
}

#[test]
fn global_outer_bank_sparse_establishment_work_is_bounded() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    fixture
        .source
        .set_limits(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(128, 128, 128, 128, 128, 128, 128),
                64_000_000,
                64_000_000,
                10_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let (first, first_work) = fixture.global();
    let before = first_work.spent.load(Ordering::Relaxed);
    let small = establish(
        &fixture,
        None,
        first,
        fixture.documents[0],
        &first_work,
        &cancel,
    )
    .unwrap();
    let small_steps = first_work.spent.load(Ordering::Relaxed) - before;
    let mut dense = small.clone();
    drop(first_work);
    let mut final_global = None;
    for index in 0..64 {
        let (last, last_work) = fixture.global();
        if index < 63 {
            dense = establish(
                &fixture,
                Some(&dense),
                last,
                fixture.documents[0],
                &last_work,
                &cancel,
            )
            .unwrap();
        } else {
            final_global = Some((last, last_work));
        }
    }
    let (last, last_work) = final_global.unwrap();
    let before = last_work.spent.load(Ordering::Relaxed);
    let sparse = establish(
        &fixture,
        None,
        last,
        fixture.documents[1],
        &last_work,
        &cancel,
    )
    .unwrap();
    let sparse_steps = last_work.spent.load(Ordering::Relaxed) - before;
    let before = last_work.spent.load(Ordering::Relaxed);
    let extended = establish(
        &fixture,
        Some(&dense),
        last,
        fixture.documents[1],
        &last_work,
        &cancel,
    )
    .unwrap();
    let extension_steps = last_work.spent.load(Ordering::Relaxed) - before;
    assert_eq!(sparse.document(last).unwrap(), fixture.documents[1]);
    assert!(sparse.document(first).is_err());
    assert_eq!(small.document(first).unwrap(), fixture.documents[0]);
    assert_eq!(extended.document(first).unwrap(), fixture.documents[0]);
    assert_eq!(extended.document(last).unwrap(), fixture.documents[1]);
    assert!(dense.document(last).is_err());
    assert!(
        sparse_steps <= small_steps + 24 && extension_steps <= small_steps + 24,
        "one Global establishment spent {sparse_steps} steps across 64 holes and {extension_steps} after 64 unrelated rows, versus {small_steps} without either"
    );
}
