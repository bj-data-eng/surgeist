//! Identity-index admission only. Native Documents are genuine payload IDs;
//! these fixtures do not certify hosted Documents, agents, loops or services.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn cancellation() -> AtomicBool {
    AtomicBool::new(false)
}

pub(in crate::store) struct Fixture {
    pub(in crate::store) dom: Dom,
    pub(in crate::store) source: DomHostedSource,
    host: DomHostedHostId,
    pub(in crate::store) documents: [DocumentId; 2],
}

impl Fixture {
    fn new() -> Self {
        let cancel = cancellation();
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                4096,
                64,
                0,
                0,
                16,
                4_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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

    pub(in crate::store) fn new_for_outer_rows(rows: usize) -> Self {
        let mut fixture = Self::new();
        let count = rows.checked_add(8).unwrap().max(64);
        let bytes = count
            .checked_mul(65_536)
            .unwrap()
            .checked_add(16_000_000)
            .unwrap();
        fixture
            .source
            .set_limits(
                DomHostedLimits::new(
                    DomHostedRecordLimits::new(count, count, count, count, count, count, count),
                    bytes,
                    bytes,
                    u64::try_from(count)
                        .unwrap()
                        .checked_mul(4096)
                        .unwrap()
                        .checked_add(10_000_000)
                        .unwrap(),
                ),
                &cancellation(),
            )
            .unwrap();
        fixture
    }

    pub(in crate::store) fn global(&mut self) -> (DomHostedGlobalId, Arc<Work>) {
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

fn physical_union(graphs: &[&GlobalDocumentIndex]) -> usize {
    let mut objects = std::collections::BTreeMap::new();
    for graph in graphs {
        for (id, usage) in graph.inspect_materializations().unwrap() {
            if let Some(old) = objects.insert(id, usage) {
                assert_eq!(old, usage);
            }
        }
    }
    objects.values().map(|usage| usage.bytes()).sum()
}

#[test]
fn global_index_replacement_admits_old_and_new_actual_backing_at_the_same_time() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (first, work) = fixture.global();
    let old = establish(&fixture, None, first, fixture.documents[0], &work, &cancel).unwrap();
    drop(work);
    let (next, work) = fixture.global();
    let baseline = fixture.used();
    let candidate = establish(
        &fixture,
        Some(&old),
        next,
        fixture.documents[1],
        &work,
        &cancel,
    )
    .unwrap();
    let additional = physical_union(&[&old, &candidate]) - physical_union(&[&old]);
    assert_eq!(fixture.used(), baseline + additional);
    assert_eq!(candidate.document(first), Ok(fixture.documents[0]));
    assert_eq!(old.document(next), Err(DomHostedError::StaleIdentity));
    drop(candidate);
    assert_eq!(fixture.used(), baseline);
    let limits = fixture.source.limits();
    fixture.limits(
        baseline + additional - 1,
        limits.scratch_bytes(),
        limits.work(),
    );
    assert!(matches!(
        establish(
            &fixture,
            Some(&old),
            next,
            fixture.documents[1],
            &work,
            &cancel
        ),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            ..
        })
    ));
    assert_eq!(fixture.used(), baseline);
    fixture.limits(baseline + additional, limits.scratch_bytes(), limits.work());
    let exact = establish(
        &fixture,
        Some(&old),
        next,
        fixture.documents[1],
        &work,
        &cancel,
    )
    .unwrap();
    assert_eq!(fixture.used(), baseline + additional);
    assert_eq!(exact.document(next), Ok(fixture.documents[1]));
    drop(exact);
    assert_eq!(fixture.used(), baseline);
}

#[test]
fn global_index_scratch_prices_capacity_and_header_not_only_present_mappings() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (first, work) = fixture.global();
    drop(work);
    let (_, work) = fixture.global();
    drop(work);
    let (last, work) = fixture.global();
    let baseline = fixture.used();
    let roomy = establish(&fixture, None, last, fixture.documents[1], &work, &cancel).unwrap();
    let actual = physical_union(&[&roomy]);
    assert_eq!(fixture.used(), baseline + actual);
    assert_eq!(roomy.document(first), Err(DomHostedError::StaleIdentity));
    assert_eq!(roomy.document(last), Ok(fixture.documents[1]));
    drop(roomy);
    assert_eq!(fixture.used(), baseline);
    let limits = fixture.source.limits();
    fixture.limits(limits.retained_bytes(), actual - 1, limits.work());
    assert!(matches!(
        establish(&fixture, None, last, fixture.documents[1], &work, &cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::ScratchBytes,
            ..
        })
    ));
    assert_eq!(fixture.used(), baseline);
    fixture.limits(limits.retained_bytes(), actual, limits.work());
    let exact = establish(&fixture, None, last, fixture.documents[1], &work, &cancel).unwrap();
    assert_eq!(fixture.used(), baseline + actual);
    assert_eq!(exact.document(last), Ok(fixture.documents[1]));
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
    let spent = work.spent.load(Ordering::Relaxed);
    let limits = fixture.source.limits();
    let limit = spent + 3;
    fixture.limits(limits.retained_bytes(), limits.scratch_bytes(), limit);
    assert!(
        matches!(establish(&fixture,Some(&old),next,fixture.documents[1],&work,&cancel),Err(DomHostedError::WorkLimit {limit:actual}) if actual==limit)
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), limit);
    assert_eq!(fixture.used(), baseline);
    assert_eq!(old.document(first), Ok(fixture.documents[0]));
    assert_eq!(old.document(next), Err(DomHostedError::StaleIdentity));
    fixture.limits(
        limits.retained_bytes(),
        limits.scratch_bytes(),
        limits.work(),
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
    assert!(work.spent.load(Ordering::Relaxed) > spent);
    assert_eq!(old.document(global), Ok(fixture.documents[0]));
}

#[test]
fn global_index_foreign_issued_identity_remains_a_stale_nonmember() {
    let cancel = cancellation();
    let mut fixture = Fixture::new();
    let (global, work) = fixture.global();
    let old = establish(&fixture, None, global, fixture.documents[0], &work, &cancel).unwrap();
    let mut foreign = Fixture::new();
    let (other, _other_work) = foreign.global();
    assert_eq!(old.document(global), Ok(fixture.documents[0]));
    assert_eq!(old.document(other), Err(DomHostedError::StaleIdentity));
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
