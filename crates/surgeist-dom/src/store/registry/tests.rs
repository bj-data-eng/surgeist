//! Canonical registry-bank admission, without hosted execution or service claims.

use super::*;
use crate::hosted::{DomHostedLimits, DomHostedRecordLimits, DomHostedSourceId};
use std::sync::atomic::Ordering;

#[test]
fn registry_replacement_refuses_candidate_backing_above_scratch_even_when_producer_fits() {
    let cancel = AtomicBool::new(false);
    let formed = HtmlCapture::complete_empty_global_registry_with(
        HtmlHostFactsRevision::fresh(),
        usize::MAX,
        |_| Ok::<_, ()>(()),
    )
    .unwrap();
    let producer_peak = formed.formation_peak_bytes();
    drop(formed);
    // Establish that the only reduced allowance still admits the actual
    // complete-empty producer. The bank below has a separate replacement Vec
    // occurrence: its header and backing must also fit scratch before reserve
    // or retained-record copies, as in the hosted identity/index owners.
    let exact = HtmlCapture::complete_empty_global_registry_with(
        HtmlHostFactsRevision::fresh(),
        producer_peak,
        |_| Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(exact.formation_peak_bytes(), producer_peak);
    drop(exact);

    let vector_header = size_of::<Vec<Arc<Record>>>();
    let record_width = size_of::<Arc<Record>>();
    assert!(producer_peak >= vector_header);
    let required = (producer_peak - vector_header) / record_width + 1;
    let candidate_requested = vector_header + required * record_width;
    assert!(candidate_requested > producer_peak);
    assert!(candidate_requested - record_width <= producer_peak);

    let owner = DomOwnerId::allocate().unwrap();
    let limits = DomHostedLimits::new(
        DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, required),
        64_000_000,
        64_000_000,
        10_000_000,
    );
    let mut source =
        DomHostedSource::new(DomHostedSourceId { owner, serial: 0 }, limits, &cancel).unwrap();
    let work = source.work(&cancel).unwrap();
    let mut old: Option<Arc<Bank>> = None;
    for _ in 0..required - 1 {
        let (next, _) = Bank::form(old.as_deref(), owner, &source, &work, None, &cancel).unwrap();
        old = Some(next);
    }
    let old = old.unwrap();
    let old_length = old.records.len();
    let old_capacity = old.records.capacity();
    let first_identity = old.records[0].capture.identity();
    assert_eq!(old_length + 1, required);

    source
        .set_limits(
            DomHostedLimits::new(
                limits.records(),
                limits.retained_bytes(),
                producer_peak,
                limits.work(),
            ),
            &cancel,
        )
        .unwrap();
    let retained_before = source.limits().retained_bytes() - source.remaining_bytes().unwrap();
    let spent_before = work.spent.load(Ordering::Relaxed);
    match Bank::form(Some(&old), owner, &source, &work, None, &cancel) {
        Err(DomHostedError::Limit {
            resource: DomHostedResource::ScratchBytes,
            limit,
        }) => assert_eq!(limit, producer_peak),
        Err(error) => panic!("expected candidate scratch refusal, got {error:?}"),
        Ok((candidate, _)) => {
            let actual = vector_header + candidate.records.capacity() * record_width;
            panic!(
                "registry replacement exceeded scratch: limit={producer_peak}, \
                 requested={candidate_requested}, actual={actual}, \
                 old_len={old_length}, old_capacity={old_capacity}, \
                 candidate_capacity={}",
                candidate.records.capacity(),
            );
        }
    }
    // A refused candidate cannot replace the old graph or leave a retained
    // debit behind. Work already spent inspecting/forming it is not refunded.
    assert_eq!(old.records.len(), old_length);
    assert_eq!(old.records.capacity(), old_capacity);
    assert_eq!(old.records[0].capture.identity(), first_identity);
    assert_eq!(
        source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
        retained_before,
    );
    assert!(work.spent.load(Ordering::Relaxed) > spent_before);
}
