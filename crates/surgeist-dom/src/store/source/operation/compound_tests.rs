//! Original first-error custody through public refusal, Drop, resume and cancel.
use super::*;
use crate::*;

fn fixture() -> (Dom, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let bounds = DomLimits::new(
        32,
        4096,
        64,
        0,
        0,
        8,
        100_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap();
    let mut dom = Dom::new(bounds).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let child = batch.create_element(&document, "root");
    batch.insert_before(&child, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&child).unwrap().unwrap(),
    )
}
fn policy(work: u64, phases: u64) -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, work, phases, 4096, 4096, 8)
}
fn edit(document: DocumentId, child: NodeId) -> DomSourceEdit {
    DomSourceEdit::append(
        document.node(),
        vec![child.into(), DocumentString::from("x").into()],
    )
}
fn assert_failed_once(dom: &mut Dom, operation: DomSourceOperationId, child: NodeId) {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .resume_source_edit(operation, policy(20_000_000, 16), &cancel)
        .unwrap();
    lease.request_cancel();
    let DomSourceProgress::Failed(failure) = lease.advance(&cancel).unwrap() else {
        panic!("first ordinary error wins over later cancellation")
    };
    assert_eq!(failure.operation(), operation);
    assert_eq!(failure.error(), &DomError::HierarchyRequest);
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.usage().managed_nodes(), 4);
    let fragment = dom.snapshot().node(child).unwrap().parent().unwrap();
    assert_eq!(dom.snapshot().node(fragment).unwrap().children().len(), 2);
}

#[test]
fn compound_first_error_survives_final_phase_refusal_drop_resume_and_cancellation() {
    let (mut dom, document, child) = fixture();
    let cancel = AtomicBool::new(false);
    let before = dom.snapshot().stamp();
    let operation = {
        let mut lease = dom
            .begin_source_edit(edit(document, child), policy(20_000_000, 0), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::SourcePublicationLimit)
        ));
        assert_eq!(lease.snapshot().stamp(), before);
        lease.operation()
    };
    let status = dom.pending_source_edit().unwrap();
    assert!(!status.has_published());
    assert_eq!(status.phase(), DomSourcePhase::PendingCleanup);
    assert_failed_once(&mut dom, operation, child);
    assert_ne!(dom.snapshot().stamp(), before);
}

#[test]
fn work_refusal_after_first_error_keeps_the_same_public_invocation_and_error() {
    let cancel = AtomicBool::new(false);
    // Measure only the finite search extent. Work-step counts are deliberately
    // not an oracle; representation changes may move the cut while the public
    // first-error, identity, tree and cancellation outcomes stay the same.
    let ceiling = {
        let (mut dom, document, child) = fixture();
        let (operation, spent) = {
            let mut lease = dom
                .begin_source_edit(edit(document, child), policy(20_000_000, 0), &cancel)
                .unwrap();
            assert!(matches!(
                lease.advance(&cancel),
                Err(DomError::SourcePublicationLimit)
            ));
            (lease.operation(), lease.status().spent_work())
        };
        assert_failed_once(&mut dom, operation, child);
        spent
    };
    let (mut lower, mut upper) = (0, ceiling);
    while lower < upper {
        let work = lower + (upper - lower) / 2;
        let (mut dom, document, child) = fixture();
        let mut lease = match dom.begin_source_edit(edit(document, child), policy(work, 0), &cancel)
        {
            Ok(lease) => lease,
            Err(DomError::SourceWorkLimit { .. }) => {
                lower = work + 1;
                continue;
            }
            Err(error) => panic!("actual Source entry: {error:?}"),
        };
        let result = lease.advance(&cancel);
        let status = lease.status();
        if matches!(result, Err(DomError::SourceWorkLimit { .. }))
            && status.phase() == DomSourcePhase::PendingCleanup
        {
            let operation = lease.operation();
            drop(lease);
            assert!(!dom.pending_source_edit().unwrap().has_published());
            assert_failed_once(&mut dom, operation, child);
            return;
        }
        if status.phase() == DomSourcePhase::PendingCleanup {
            upper = work;
        } else {
            lower = work + 1;
        }
    }
    panic!("a work cut after the original first error must retain its cleanup");
}
