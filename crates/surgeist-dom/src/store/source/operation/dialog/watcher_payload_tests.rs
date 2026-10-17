//! Relative Hosted runtime payload on the actual original Source/ancestor engine.
//! No Window, watcher identity or Runtime execution receipt is fabricated here.
use super::admission_tests::update_limits;
use super::tests::{attest, fixture, limits, request};
use super::*;
struct Allocation {
    values: Vec<NodeId>,
    _payload: crate::DomHostedRuntimeRetention,
}
fn issuer(dom: &mut Dom) -> crate::DomHostedSource {
    dom.register_hosted_source(
        crate::DomHostedLimits::new(
            crate::DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
            16_000_000,
            16_000_000,
            4_000_000,
        ),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn extent(count: usize) -> usize {
    size_of::<Allocation>() + count * size_of::<NodeId>()
}
fn allocation(
    dom: &mut Dom,
    source: &crate::DomHostedSource,
    target: NodeId,
    count: usize,
) -> Result<Allocation, DomError> {
    let payload = source.retain_runtime_payload(dom, extent(count), &AtomicBool::new(false))?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).unwrap();
    assert_eq!(
        values.capacity(),
        count,
        "independently concrete owning bank capacity"
    );
    values.resize(count, target);
    Ok(Allocation {
        values,
        _payload: payload,
    })
}
#[test]
fn hosted_runtime_allocation_exact_and_one_short_preserve_original_source_owner() {
    let cancel = AtomicBool::new(false);
    for short in [true, false] {
        let (mut dom, _executor, _, first, _) = fixture();
        let source = issuer(&mut dom);
        let event = request(&mut dom, first);
        let baseline = dom
            .source_observation_total(storage::Payload::default(), &cancel)
            .unwrap();
        let mut tight = limits();
        tight.retained_bytes = baseline.bytes + extent(13) - usize::from(short);
        update_limits(&mut dom, &event, tight);
        let before = dom.snapshot().stamp();
        match allocation(&mut dom, &source, first, 13) {
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit,
            }) if short => assert_eq!(limit, tight.retained_bytes),
            Ok(owned) if !short => {
                assert_eq!(owned.values.len(), 13);
                assert_eq!(
                    source.shared.runtime_bytes.load(Ordering::Relaxed),
                    extent(13)
                );
                assert_eq!(
                    dom.source_observation_total(storage::Payload::default(), &cancel)
                        .unwrap()
                        .bytes,
                    baseline.bytes + extent(13)
                );
                drop(owned);
            }
            other => panic!(
                "exact real Source payload admission disagreed: {}",
                if other.is_ok() {
                    "unexpected admission"
                } else {
                    "unexpected refusal"
                }
            ),
        }
        assert_eq!(source.shared.runtime_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(
            dom.source_observation_total(storage::Payload::default(), &cancel)
                .unwrap()
                .bytes,
            baseline.bytes
        );
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            event.operation()
        );
    }
}
#[test]
fn hosted_runtime_allocation_survives_child_completion_and_releases_under_same_parent() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let source = issuer(&mut dom);
    let parent = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &parent, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let child = request(&mut dom, second);
    let owned = allocation(&mut dom, &source, first, 17).unwrap();
    attest(&mut dom, &executor, &child, DomSourceEventOutcome::Canceled);
    let result = dom
        .resume_source_edit(child.operation(), limits(), &cancel)
        .unwrap()
        .advance(&cancel)
        .unwrap();
    assert!(matches!(result, DomSourceProgress::Complete(_)));
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        parent.operation()
    );
    assert_eq!(
        source.shared.runtime_bytes.load(Ordering::Relaxed),
        extent(17)
    );
    let retained = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap()
        .bytes;
    drop(owned);
    let released = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap()
        .bytes;
    assert_eq!(retained - released, extent(17));
    assert_eq!(source.shared.runtime_bytes.load(Ordering::Relaxed), 0);
    drop(scope);
}
