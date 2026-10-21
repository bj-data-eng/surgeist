//! Child constructor payload admission through the original Source formation.
//! Closed native Document metadata supplies real operands; no hosted Global or
//! completed activity state is synthesized by these private boundary fixtures.
use super::*;
use crate::DomSourceLimits;
use crate::document::{HostedDocumentInputData, HostedDocumentInputs};
use source::formation::Formation;
use surgeist_html_semantics::HtmlBaseUrlRecord;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn hosted_limits(bytes: usize, scratch: usize) -> DomHostedLimits {
    DomHostedLimits::new(
        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
        bytes,
        scratch,
        1_000_000,
    )
}
fn source_limits(bytes: usize, units: usize, work: u64) -> DomSourceLimits {
    DomSourceLimits::new(bytes, units, work, 64, 4096, 4096, 1)
}
struct Fixture {
    dom: Dom,
    source: DomHostedSource,
    lease: DomHostedCreationLease,
    work: Arc<Work>,
    parent: DomDocumentMetadata,
    source_work: Arc<AtomicU64>,
    source_work_id: crate::store::SourceWorkIdentity,
}
impl Fixture {
    fn new() -> Self {
        let c = cancel();
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
        let document = batch.create_document();
        let document = dom.apply(batch, &c).unwrap().document(&document).unwrap();
        let mut parent = dom
            .snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .clone();
        let mut source = dom
            .register_hosted_source(hosted_limits(8_000_000, 8_000_000), &c)
            .unwrap();
        let host = source.issue_host(&c).unwrap();
        let mut prefix = Some(source.prepare_top_level(host, &c).unwrap());
        let mut prepared = Some(source.seal_top_level(&mut prefix, &c).unwrap());
        let lease = dom
            .begin_hosted_top_level(&mut source, &mut prepared, &c)
            .unwrap();
        let work = dom
            .hosted_operation(lease.operation)
            .unwrap()
            .request()
            .work
            .clone();
        // These once-formed values are constructor operands only. The actual
        // closed Document does not acquire hosted environment/activity proof.
        parent.hosted_inputs =
            Some(HostedDocumentInputs::form(&source, &work, parent.origin(), &c).unwrap());
        Self {
            dom,
            source,
            lease,
            work,
            parent,
            source_work_id: crate::store::SourceWorkIdentity::fresh().unwrap(),
            source_work: Arc::new(AtomicU64::new(0)),
        }
    }
    fn draft(&self) -> Draft {
        Draft::new_hosted_source(
            &self.dom,
            DomBatch::new().owner,
            source_limits(16_000_000, 1_000_000, 1_000_000),
            self.source_work.clone(),
            self.source_work_id,
            self.lease.operation,
            &cancel(),
        )
        .unwrap()
    }
    fn form(
        &self,
        formation: &Formation,
        c: &AtomicBool,
    ) -> Result<HostedDocumentInputs, DomHostedError> {
        let parent_inputs = self.parent.hosted_inputs.as_ref().unwrap();
        HostedDocumentInputs::form_child(
            &self.source,
            &self.work,
            ChildConstructor {
                parent: &self.parent,
                creator_base: parent_inputs.0.facts.document_url().unwrap(),
                top_policy: &parent_inputs.0.opener,
                sandbox: surgeist_html::HtmlSandboxFlags::empty(),
            },
            Some(formation),
            c,
        )
    }
    fn used(&self) -> usize {
        self.source.limits().retained_bytes() - self.source.remaining_bytes().unwrap()
    }
    fn expected_payload(&self) -> (usize, usize) {
        // Fixed recipe: new intrinsic about:blank record, separate creator-base
        // and top-creation record occurrences, and the actual referrer backing.
        // The producer prices its own opaque record; no private URL layout or
        // constructor result receipt is used to manufacture an expectation.
        let literal =
            DocumentString::from_utf16(vec![97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107]);
        let new = HtmlBaseUrlRecord::observed_storage_usage(&literal).unwrap();
        let parent = self.parent.hosted_inputs.as_ref().unwrap();
        let base = parent
            .0
            .facts
            .document_url()
            .unwrap()
            .retained_storage_usage();
        let referrer = self.parent.url().shared_code_units().capacity();
        (
            size_of::<HostedDocumentInputData>()
                + new.retained_bytes()
                + 2 * base.retained_bytes()
                + size_of::<Vec<u16>>()
                + referrer * size_of::<u16>(),
            new.utf16_extent() + 2 * base.utf16_extent() + referrer,
        )
    }
}
fn base(draft: &Draft) -> (usize, usize) {
    (
        draft.source_storage_bytes(&cancel()).unwrap() + draft.source_other_bytes,
        draft.source_storage_units(&cancel()).unwrap() + draft.source_other_units,
    )
}
fn assert_released(draft: &Draft, fixture: &Fixture, original: usize) {
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    assert_eq!(fixture.used(), original);
}

#[test]
fn operation_bank_source_formation_admits_its_inline_candidate_header_once() {
    let mut f = Fixture::new();
    let before_credit = f.used();
    let credit = f.source.retain(size_of::<Operation>(), &cancel()).unwrap();
    let mut draft = f.draft();
    let (base_bytes, base_units) = base(&draft);
    let original = f.used();
    let resident_storage = f.dom.hosted_operations.inspect_storage_bytes();
    let source_revision = f.source.revision().unwrap();
    let canonical_revision = f.dom.revision;
    let slot = f.source.next_operation_slot(&f.work, &cancel()).unwrap();
    assert_eq!(slot, f.lease.operation.slot + 1);
    // This is the original canonical bank and issuer Work. A genuine native
    // Document supplies the Draft payload; no hosted Global or activity state
    // is invented to exercise this preparation boundary.
    let backing = {
        let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
        let prepared = f
            .dom
            .hosted_operations
            .prepare(
                slot,
                &f.work,
                &f.source,
                0,
                Some(&formation),
                operations::BodyCredit::Borrowed(&credit),
                operations::Projection::default(),
                &cancel(),
            )
            .unwrap();
        let backing = prepared.inspect_new_storage_bytes();
        assert!(backing > 0, "the actual new empty destination path");
        drop(prepared);
        drop(formation);
        assert_released(&draft, &f, original);
        assert_eq!(
            f.dom.hosted_operations.inspect_storage_bytes(),
            resident_storage
        );
        backing
    };
    // The one PreparedRows header includes its empty-path owner. Its actual
    // newly allocated pages, Slot and holder excess remain out of line; resident
    // canonical rows are neither copied nor charged as another occurrence.
    let header = size_of::<operations::PreparedRows>();
    let exact = header + backing;
    for allowance in [exact - 1, exact] {
        draft.source_limits = Some(source_limits(base_bytes + allowance, base_units, 1_000_000));
        assert_eq!(
            draft.source_limits.unwrap().retained_bytes,
            base_bytes + allowance
        );
        let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
        let source_work = f.source_work.load(Ordering::Relaxed);
        let hosted_work = f.work.spent.load(Ordering::Relaxed);
        let result = f.dom.hosted_operations.prepare(
            slot,
            &f.work,
            &f.source,
            0,
            Some(&formation),
            operations::BodyCredit::Borrowed(&credit),
            operations::Projection::default(),
            &cancel(),
        );
        if allowance < exact {
            match result {
                Err(DomHostedError::Dom(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit,
                })) if limit == base_bytes + allowance => {}
                Ok(_) => panic!(
                    "one-short Source formation admitted a bank: base={base_bytes}, header={header}, backing={backing}, exact={exact}"
                ),
                Err(error) => {
                    panic!("one-short Source formation returned an unrelated error: {error:?}")
                }
            }
        } else {
            let prepared = match result {
                Ok(prepared) => prepared,
                Err(error) => panic!(
                    "exact physical Source formation refused: base={base_bytes}, header={header}, backing={backing}, exact={exact}, error={error:?}"
                ),
            };
            assert_eq!(prepared.inspect_new_storage_bytes(), backing);
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), exact);
            drop(prepared);
        }
        assert!(f.source_work.load(Ordering::Relaxed) > source_work);
        assert!(f.work.spent.load(Ordering::Relaxed) > hosted_work);
        drop(formation);
        assert_released(&draft, &f, original);
        assert_eq!(
            f.dom.hosted_operations.inspect_storage_bytes(),
            resident_storage
        );
        assert_eq!(f.source.revision().unwrap(), source_revision);
        assert_eq!(f.dom.revision, canonical_revision);
        assert!(Arc::ptr_eq(
            &f.dom
                .hosted_operations
                .get(f.lease.operation)
                .unwrap()
                .request()
                .work,
            &f.work,
        ));
    }
    // The original borrowing guard temporarily owns the actual Policy row.
    // Its slot remains in the same allocated bank, so preparing that now-empty
    // slot exercises the no-growth case without a replacement owner or issuer.
    for allowance in [header - 1, header] {
        draft.source_limits = Some(source_limits(base_bytes + allowance, base_units, 1_000_000));
        let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
        let source = &f.source;
        let work = &f.work;
        f.dom
            .with_hosted_operation(f.lease.operation, |dom, operation| {
                let result = dom.hosted_operations.prepare(
                    operation.request().key.operation.slot,
                    work,
                    source,
                    0,
                    Some(&formation),
                    operations::BodyCredit::Borrowed(&operation._retained),
                    operations::Projection::default(),
                    &cancel(),
                );
                if allowance < header {
                    assert!(matches!(result,
                    Err(DomHostedError::Dom(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes, limit,
                    })) if limit == base_bytes + allowance));
                } else {
                    let prepared = result.unwrap();
                    assert_eq!(prepared.inspect_new_storage_bytes(), 0);
                    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), header);
                    drop(prepared);
                }
                Ok(operations::OperationResult::Retain(()))
            })
            .unwrap();
        drop(formation);
        assert_released(&draft, &f, original);
        assert_eq!(
            f.dom.hosted_operations.inspect_storage_bytes(),
            resident_storage
        );
        assert_eq!(f.source.revision().unwrap(), source_revision);
        assert_eq!(f.dom.revision, canonical_revision);
        assert!(Arc::ptr_eq(
            &f.dom
                .hosted_operations
                .get(f.lease.operation)
                .unwrap()
                .request()
                .work,
            &f.work,
        ));
    }
    drop(credit);
    assert_eq!(f.used(), before_credit);
}

#[test]
fn child_source_requested_and_actual_byte_peaks_refuse_one_short_and_retry_exactly() {
    let f = Fixture::new();
    let mut draft = f.draft();
    let (base_bytes, base_units) = base(&draft);
    let original = f.used();
    let (bytes, units) = f.expected_payload();
    let workspace = size_of::<ConstructorFormation>();
    let requested = workspace + size_of::<HostedDocumentInputData>() + size_of::<Vec<u16>>() + 22;
    assert!(workspace + bytes > requested);
    for allowance in [requested - 1, workspace + bytes - 1] {
        draft.source_limits = Some(source_limits(
            base_bytes + allowance,
            base_units + units,
            1_000_000,
        ));
        let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
        let before = f.source_work.load(Ordering::Relaxed);
        assert!(matches!(f.form(&formation, &cancel()),
            Err(DomHostedError::Dom(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit })) if limit == base_bytes + allowance));
        assert!(f.source_work.load(Ordering::Relaxed) > before);
        drop(formation);
        assert_released(&draft, &f, original);
    }
    draft.source_limits = Some(source_limits(
        base_bytes + workspace + bytes,
        base_units + units,
        1_000_000,
    ));
    let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
    let inputs = f.form(&formation, &cancel()).unwrap();
    assert_eq!(formation.owned_payload().bytes, bytes);
    assert_eq!(formation.owned_payload().units, units);
    assert_eq!(inputs.payload_bytes().unwrap(), bytes);
    assert_eq!(inputs.payload_units().unwrap(), units);
    assert_eq!(f.used(), original + bytes);
    assert_eq!(
        inputs.url().serialization(),
        &DocumentString::from("about:blank")
    );
    drop(inputs);
    drop(formation);
    assert_released(&draft, &f, original);
}

#[test]
fn child_source_utf16_and_hosted_scratch_one_short_release_every_partial_occurrence() {
    let mut f = Fixture::new();
    let mut draft = f.draft();
    let (base_bytes, base_units) = base(&draft);
    let (bytes, units) = f.expected_payload();
    let original = f.used();
    let scratch = size_of::<ConstructorFormation>() + bytes;
    for allowance in [10, units - 1] {
        draft.source_limits = Some(source_limits(
            base_bytes + scratch,
            base_units + allowance,
            1_000_000,
        ));
        let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
        assert!(matches!(f.form(&formation, &cancel()),
            Err(DomHostedError::Dom(DomError::ResourceLimit { resource: DomResource::SourceRetainedUnits, limit })) if limit == base_units + allowance));
        drop(formation);
        assert_released(&draft, &f, original);
    }
    draft.source_limits = Some(source_limits(
        base_bytes + scratch,
        base_units + units,
        1_000_000,
    ));
    f.source
        .set_limits(hosted_limits(8_000_000, scratch - 1), &cancel())
        .unwrap();
    let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
    assert!(matches!(f.form(&formation, &cancel()),
        Err(DomHostedError::Limit { resource: DomHostedResource::ScratchBytes, limit }) if limit == scratch - 1));
    drop(formation);
    assert_released(&draft, &f, original);
    f.source
        .set_limits(hosted_limits(8_000_000, scratch), &cancel())
        .unwrap();
    let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
    let inputs = f.form(&formation, &cancel()).unwrap();
    drop(inputs);
    drop(formation);
    assert_released(&draft, &f, original);
}

#[test]
fn child_source_cancellation_and_spent_work_keep_original_counters_for_raised_retry() {
    let f = Fixture::new();
    let mut draft = f.draft();
    let original = f.used();
    // Determine only the setup scan cost, not the constructor tariff. The
    // independent cut permits the constructor's entry then refuses its first
    // Source payload admission on the same original counter.
    let before_setup = f.source_work.load(Ordering::Relaxed);
    drop(Formation::new(&draft, &cancel()).unwrap().unwrap());
    let setup = f.source_work.load(Ordering::Relaxed) - before_setup;
    let limit = f.source_work.load(Ordering::Relaxed) + setup;
    draft.source_limits = Some(source_limits(16_000_000, 1_000_000, limit));
    let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
    let hosted_before = f.work.spent.load(Ordering::Relaxed);
    assert!(matches!(f.form(&formation, &cancel()),
        Err(DomHostedError::Dom(DomError::SourceWorkLimit { limit: actual })) if actual == limit));
    assert_eq!(f.source_work.load(Ordering::Relaxed), limit);
    assert!(f.work.spent.load(Ordering::Relaxed) > hosted_before);
    drop(formation);
    assert_released(&draft, &f, original);

    draft.source_limits = Some(source_limits(16_000_000, 1_000_000, 1_000_000));
    let formation = Formation::new(&draft, &cancel()).unwrap().unwrap();
    let source_before = f.source_work.load(Ordering::Relaxed);
    let hosted_before = f.work.spent.load(Ordering::Relaxed);
    assert!(matches!(
        f.form(&formation, &AtomicBool::new(true)),
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));
    assert_eq!(f.source_work.load(Ordering::Relaxed), source_before);
    assert_eq!(f.work.spent.load(Ordering::Relaxed), hosted_before);
    assert_released(&draft, &f, original);
    let inputs = f.form(&formation, &cancel()).unwrap();
    assert!(f.source_work.load(Ordering::Relaxed) > source_before);
    assert!(f.work.spent.load(Ordering::Relaxed) > hosted_before);
    assert!(Arc::ptr_eq(draft.source_work.source_arc(), &f.source_work));
    assert!(Arc::ptr_eq(
        &f.work,
        &f.dom
            .hosted_operation(f.lease.operation)
            .unwrap()
            .request()
            .work
    ));
    drop(inputs);
    drop(formation);
    assert_released(&draft, &f, original);
}
