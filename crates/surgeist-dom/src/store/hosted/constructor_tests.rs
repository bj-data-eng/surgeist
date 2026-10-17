//! Actual constructor formation and original policy/operation custody. No
//! external agent, Global or completed hosted Document is fabricated here.
use super::*;
use crate::document::{HostedDocumentInputData, HostedDocumentInputs};
use surgeist_html_semantics::HtmlBaseUrlRecord;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(bytes: usize, scratch: usize, work: u64) -> DomHostedLimits {
    DomHostedLimits::new(
        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
        bytes,
        scratch,
        work,
    )
}
fn prepared_top_level() -> (Dom, DomHostedSource, Option<DomHostedPreparedTopLevel>) {
    let c = cancel();
    let mut dom = Dom::new(DomLimits::new(64, 4096, 64, 0, 0, 16, 4_000_000).unwrap()).unwrap();
    let mut source = dom
        .register_hosted_source(limits(8_000_000, 8_000_000, 1_000_000), &c)
        .unwrap();
    let host = source.issue_host(&c).unwrap();
    let mut prefix = Some(source.prepare_top_level(host, &c).unwrap());
    let prepared = Some(source.seal_top_level(&mut prefix, &c).unwrap());
    (dom, source, prepared)
}

#[test]
fn hosted_top_level_scratch_admits_the_final_policy_operation_before_issuance() {
    let c = cancel();
    let capacity = {
        let (dom, source, prepared) = prepared_top_level();
        let work = &prepared.as_ref().unwrap().preparation.work;
        let bank = dom
            .hosted_operations
            .prepare(
                source.next_operation_slot(work, &c).unwrap(),
                work,
                &source,
                0,
                None,
                &c,
            )
            .unwrap();
        bank.candidate_capacity().unwrap()
    };
    // Operation and PreparedRows are simultaneous arguments to the real
    // canonical insertion. PreparedRows includes its candidate owner inline;
    // only the replacement row backing is additional.
    let row_backing = capacity * size_of::<Option<Box<Operation>>>();
    let exact = size_of::<Operation>() + size_of::<operations::PreparedRows>() + row_backing;
    for allowance in [exact, exact - 1] {
        let (mut dom, mut source, mut prepared) = prepared_top_level();
        let limits_before = source.limits();
        source
            .set_limits(
                limits(
                    limits_before.retained_bytes(),
                    allowance,
                    limits_before.work(),
                ),
                &c,
            )
            .unwrap();
        let work = prepared.as_ref().unwrap().preparation.work.clone();
        let prefix = std::ptr::from_ref(prepared.as_ref().unwrap());
        let retained = used(&source);
        let source_revision = source.revision().unwrap();
        let canonical_revision = dom.revision;
        let reserved = dom.hosted_revisions;
        let spent = work.spent.load(Ordering::Relaxed);
        match dom.begin_hosted_top_level(&mut source, &mut prepared, &c) {
            Ok(lease) if allowance == exact => {
                assert!(prepared.is_none());
                assert_eq!(dom.hosted_operations.row_capacity(), capacity);
                let DomHostedCreationStep::PermissionsPolicy(request) = lease.step(&dom).unwrap()
                else {
                    panic!("actual Policy operation")
                };
                assert!(Arc::ptr_eq(&request.request.work, &work));
                source
                    .observe()
                    .validate(
                        lease.operation.source,
                        Role::Operation,
                        lease.operation.slot,
                        lease.operation.generation,
                    )
                    .unwrap();
                assert_eq!(dom.revision, canonical_revision);
                assert_eq!(dom.hosted_revisions, reserved + 5);
                eprintln!(
                    "exact Policy installation succeeded: operation_header={}, prepared_header={}, row_backing={row_backing}, exact={exact}",
                    size_of::<Operation>(),
                    size_of::<operations::PreparedRows>()
                );
            }
            Err(DomHostedError::Limit {
                resource: DomHostedResource::ScratchBytes,
                limit,
            }) if allowance < exact && limit == allowance => {
                assert_eq!(std::ptr::from_ref(prepared.as_ref().unwrap()), prefix);
                assert_eq!(used(&source), retained);
                assert_eq!(source.revision().unwrap(), source_revision);
                assert_eq!(dom.revision, canonical_revision);
                assert_eq!(dom.hosted_revisions, reserved);
            }
            Ok(lease) => panic!(
                "one-short final Policy payload admitted: operation={:?}, operation_header={}, prepared_header={}, row_backing={row_backing}, exact={exact}",
                lease.operation,
                size_of::<Operation>(),
                size_of::<operations::PreparedRows>()
            ),
            Err(error) => panic!(
                "Policy installation returned an unrelated error at allowance {allowance}: {error:?}"
            ),
        }
        assert!(work.spent.load(Ordering::Relaxed) > spent);
    }
}
fn fixture() -> (
    Dom,
    DomHostedSource,
    DomHostedCreationLease,
    Option<DomHostedPolicyPublication>,
) {
    let c = cancel();
    let mut dom = Dom::new(DomLimits::new(64, 4096, 64, 0, 0, 16, 4_000_000).unwrap()).unwrap();
    let mut source = dom
        .register_hosted_source(limits(8_000_000, 8_000_000, 1_000_000), &c)
        .unwrap();
    let host = source.issue_host(&c).unwrap();
    let mut prefix = Some(source.prepare_top_level(host, &c).unwrap());
    let mut prepared = Some(source.seal_top_level(&mut prefix, &c).unwrap());
    let lease = dom
        .begin_hosted_top_level(&mut source, &mut prepared, &c)
        .unwrap();
    let DomHostedCreationStep::PermissionsPolicy(request) = lease.step(&dom).unwrap() else {
        panic!("policy")
    };
    let publication = source
        .seal_permissions_policy(request, DomFullscreenInherited::Enabled, &c)
        .unwrap();
    (dom, source, lease, Some(publication))
}
fn used(source: &DomHostedSource) -> usize {
    source.limits().retained_bytes() - source.remaining_bytes().unwrap()
}

#[test]
fn incomplete_and_expired_constructor_authority_never_substitutes_false_for_activity() {
    // Exercise the canonical issuer/constructor boundary, not a claim that a
    // Runtime Global or completed initial entry has been constructed. Every
    // phase below is reached through the original request/publication owner.
    let c = cancel();
    let (mut dom, mut source, mut lease, mut policy) = fixture();
    lease
        .accept_permissions_policy(&mut dom, &mut source, &mut policy, &c)
        .unwrap();
    let DomHostedCreationStep::NativeExecution(request) = lease.step(&dom).unwrap() else {
        panic!("actual constructor execution request")
    };
    let global = source.issue_global(request, &c).unwrap();
    let mut execution = Some(
        source
            .seal_native_execution(request, global, 100, &c)
            .unwrap(),
    );
    lease
        .accept_publication(&mut dom, &mut source, &mut execution, &c)
        .unwrap();
    assert!(execution.is_none());
    let DomHostedCreationStep::MakeActive(request) = lease.step(&dom).unwrap() else {
        panic!("actual constructed Document awaiting make-active")
    };
    let document = request.document();
    let draft = Draft::new(&dom, Arc::new(BatchIdentity), &c).unwrap();
    let key = draft.resolve(document.into()).unwrap();
    let metadata = draft.document_metadata(key).unwrap();
    let origin = metadata.origin().clone();
    let environment = metadata.document_environment();
    let association = environment.hosted.as_ref().unwrap();
    assert_eq!(association.global, global);
    assert!(matches!(association.phase, DocumentPhase::Constructed));
    assert!(association.validate_current().is_ok());
    assert_eq!(
        association.validate_live(),
        Err(DomHostedError::IncompleteOperation)
    );
    assert_eq!(
        draft.document_fully_active(key, &c),
        Err(DomError::HostedSourceUnavailable)
    );
    assert_eq!(
        dom.apply(DomBatch::new(), &c).unwrap_err(),
        DomError::SourceOperationPending
    );
    let stamp = dom.snapshot().stamp();
    drop(source);
    assert_eq!(
        association.validate_current(),
        Err(DomHostedError::SourceExpired)
    );
    assert_eq!(
        draft.document_fully_active(key, &c),
        Err(DomError::HostedSourceUnavailable)
    );
    let retained = draft.document_metadata(key).unwrap();
    assert_eq!(retained.origin(), &origin);
    assert_eq!(
        retained.document_environment().input_platform(),
        environment.input_platform()
    );
    assert_eq!(
        retained
            .document_environment()
            .hosted
            .as_ref()
            .unwrap()
            .global,
        global
    );
    assert_eq!(
        dom.apply(DomBatch::new(), &c).unwrap_err(),
        DomError::SourceOperationPending
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
}
fn expected_payload() -> usize {
    // The producer owns the record's private extent; the caller owns exactly
    // this immutable payload header. No private URL layout is copied here.
    let url = DocumentString::from_utf16(vec![97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107]);
    size_of::<HostedDocumentInputData>()
        + HtmlBaseUrlRecord::observed_storage_usage(&url)
            .unwrap()
            .retained_bytes()
}
#[test]
fn hosted_constructor_requested_and_actual_byte_refusals_keep_policy_and_release_candidate() {
    let (mut dom, mut source, mut lease, mut publication) = fixture();
    let base = used(&source);
    let policy_address = std::ptr::from_ref(publication.as_ref().unwrap());
    let work = dom
        .hosted_operation(lease.operation)
        .unwrap()
        .request()
        .work
        .clone();
    let requested_string = size_of::<Vec<u16>>() + 11 * size_of::<u16>();
    let requested = size_of::<HostedDocumentInputData>() + requested_string;
    let actual = expected_payload();
    assert!(actual > requested);
    for allowance in [requested - 1, actual - 1] {
        source
            .set_limits(limits(base + allowance, 8_000_000, 1_000_000), &cancel())
            .unwrap();
        let revision = source.revision().unwrap();
        let spent = work.spent.load(Ordering::Relaxed);
        assert!(
            matches!(lease.accept_permissions_policy(&mut dom,&mut source,&mut publication,&cancel()),
            Err(DomHostedError::Limit { resource: DomHostedResource::RetainedBytes,limit }) if limit==base+allowance)
        );
        assert_eq!(used(&source), base);
        assert_eq!(source.revision().unwrap(), revision);
        assert!(work.spent.load(Ordering::Relaxed) > spent);
        assert_eq!(
            std::ptr::from_ref(publication.as_ref().unwrap()),
            policy_address
        );
        assert!(matches!(
            lease.step(&dom).unwrap(),
            DomHostedCreationStep::PermissionsPolicy(_)
        ));
        assert!(Arc::ptr_eq(
            &work,
            &dom.hosted_operation(lease.operation)
                .unwrap()
                .request()
                .work
        ));
    }
    source
        .set_limits(limits(base + actual, 8_000_000, 1_000_000), &cancel())
        .unwrap();
    lease
        .accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel())
        .unwrap();
    assert!(publication.is_none());
    assert_eq!(used(&source), base + actual);
    let DomHostedCreationStep::NativeExecution(request) = lease.step(&dom).unwrap() else {
        panic!("execution")
    };
    let url = request.creation_url().clone();
    let encoding = request.base_document_facts().encoding().unwrap().clone();
    assert_eq!(
        url.serialization().code_units(),
        [97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107]
    );
    assert_eq!(request.top_level_creation_url(), &url);
    assert!(!request.base_document_facts().iframe_srcdoc());
    assert!(matches!(
        request.base_document_facts().about_base_url(),
        surgeist_html_semantics::HtmlAboutBaseUrl::KnownNull
    ));
    assert_eq!(
        request.opener_policy().value(),
        surgeist_html::HtmlOpenerPolicyValue::UnsafeNone
    );
    assert_eq!(
        request.opener_policy().report_only_value(),
        surgeist_html::HtmlOpenerPolicyValue::UnsafeNone
    );
    assert!(request.opener_policy().reporting_endpoint().is_none());
    assert!(
        request
            .opener_policy()
            .report_only_reporting_endpoint()
            .is_none()
    );
    let DomHostedCreationStep::NativeExecution(retry) = lease.step(&dom).unwrap() else {
        panic!("same request")
    };
    assert_eq!(retry.creation_url(), &url);
    assert_eq!(retry.base_document_facts().encoding(), Some(&encoding));
    assert!(Arc::ptr_eq(&work, &retry.request.work));
}
#[test]
fn hosted_constructor_cancellation_preserves_original_policy_and_work() {
    let (mut dom, mut source, mut lease, mut publication) = fixture();
    let bytes = used(&source);
    let revision = source.revision().unwrap();
    let work = dom
        .hosted_operation(lease.operation)
        .unwrap()
        .request()
        .work
        .clone();
    let spent = work.spent.load(Ordering::Relaxed);
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        lease.accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancelled),
        Err(DomHostedError::Dom(DomError::Cancelled))
    );
    assert!(publication.is_some());
    assert_eq!(used(&source), bytes);
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(work.spent.load(Ordering::Relaxed), spent);
    assert!(matches!(
        lease.step(&dom).unwrap(),
        DomHostedCreationStep::PermissionsPolicy(_)
    ));
    lease
        .accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel())
        .unwrap();
    assert!(publication.is_none());
}
#[test]
fn hosted_constructor_scratch_peak_and_each_work_cut_drop_actual_partial_graph() {
    let (dom, mut source, lease, _publication) = fixture();
    let work = dom
        .hosted_operation(lease.operation)
        .unwrap()
        .request()
        .work
        .clone();
    let origin = dom
        .hosted_operation(lease.operation)
        .unwrap()
        .origin
        .clone();
    let base = used(&source);
    let scratch = size_of::<ConstructorFormation>() + expected_payload();
    source
        .set_limits(limits(8_000_000, scratch - 1, 1_000_000), &cancel())
        .unwrap();
    assert!(
        matches!(HostedDocumentInputs::form(&source,&work,&origin,&cancel()),
        Err(DomHostedError::Limit { resource: DomHostedResource::ScratchBytes,limit }) if limit==scratch-1)
    );
    assert_eq!(used(&source), base);
    source
        .set_limits(limits(8_000_000, scratch, 1_000_000), &cancel())
        .unwrap();
    let before = work.spent.load(Ordering::Relaxed);
    let inputs = HostedDocumentInputs::form(&source, &work, &origin, &cancel()).unwrap();
    let cost = work.spent.load(Ordering::Relaxed) - before;
    assert_eq!(cost, 13); // entry + eleven literal UTF16 writes + observation
    assert_eq!(used(&source), base + expected_payload());
    drop(inputs);
    assert_eq!(used(&source), base);
    for allowance in 0..cost {
        let before = work.spent.load(Ordering::Relaxed);
        source
            .set_limits(limits(8_000_000, scratch, before + allowance), &cancel())
            .unwrap();
        assert!(
            matches!(HostedDocumentInputs::form(&source,&work,&origin,&cancel()),Err(DomHostedError::WorkLimit { limit }) if limit==before+allowance)
        );
        assert_eq!(used(&source), base);
        assert_eq!(work.spent.load(Ordering::Relaxed), before + allowance);
    }
}
