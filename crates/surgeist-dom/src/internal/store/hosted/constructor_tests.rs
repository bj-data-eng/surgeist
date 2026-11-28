//! Actual constructor formation and original policy/operation custody. No
//! external agent, Global or completed hosted Document is fabricated here.
use super::*;
use crate::internal::document::{HostedDocumentInputData, HostedDocumentInputs};
use surgeist_html::values::DocumentString;
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
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            64,
            0,
            0,
            16,
            4_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut source = dom
        .register_hosted_source(limits(8_000_000, 8_000_000, 1_000_000), &c)
        .unwrap();
    let host = source.issue_host(&c).unwrap();
    let mut prefix = Some(source.prepare_top_level(host, &c).unwrap());
    let prepared = Some(source.seal_top_level(&mut prefix, &c).unwrap());
    (dom, source, prepared)
}

fn publish_local_html_entry(
    dom: &mut Dom,
    source: &mut DomHostedSource,
    prepared: &mut Option<DomHostedPreparedTopLevel>,
    with_window: bool,
) -> (
    DomHostedCreationLease,
    DocumentId,
    DomHostedGlobalId,
    Option<DomHostedWindowNavigation>,
) {
    let cancel = cancel();
    let mut lease = begin_local_html_execution(dom, source, prepared, &cancel);
    let DomHostedCreationStep::NativeExecution(request) = lease.step(dom).unwrap() else {
        panic!("expected execution request");
    };
    let global = source.issue_global(request, &cancel).unwrap();
    let window = with_window.then(|| {
        source
            .create_window_navigation(request, global, &cancel)
            .unwrap()
    });
    let mut publication = Some(
        source
            .seal_native_execution(request, global, 100, window.as_ref(), &cancel)
            .unwrap(),
    );
    lease
        .accept_publication(dom, source, &mut publication, &cancel)
        .unwrap();
    let DomHostedCreationStep::MakeActive(request) = lease.step(dom).unwrap() else {
        panic!("expected MakeActive request");
    };
    let document = request.document();
    let mut publication = Some(source.seal_make_active(request, &cancel).unwrap());
    lease
        .accept_publication(dom, source, &mut publication, &cancel)
        .unwrap();
    let DomHostedCreationStep::InitialEntry(request) = lease.step(dom).unwrap() else {
        panic!("expected entry without a finish-loading request");
    };
    let entry = source.issue_entry(request, &cancel).unwrap();
    let navigable = source.issue_navigable(request, &cancel).unwrap();
    let uuid = |value: &str| {
        let mut units = [0; 36];
        for (unit, source) in units.iter_mut().zip(value.encode_utf16()) {
            *unit = source;
        }
        units
    };
    let mut initialization = Some(DomHostedEntryInitialization::new(
        uuid("00000000-0000-4000-8000-000000000001"),
        uuid("00000000-0000-4000-8000-000000000002"),
    ));
    let mut publication = Some(
        source
            .seal_initial_entry(request, entry, navigable, &mut initialization, &cancel)
            .unwrap(),
    );
    lease
        .accept_publication(dom, source, &mut publication, &cancel)
        .unwrap();
    (lease, document, global, window)
}

fn begin_local_html_execution(
    dom: &mut Dom,
    source: &mut DomHostedSource,
    prepared: &mut Option<DomHostedPreparedTopLevel>,
    cancel: &AtomicBool,
) -> DomHostedCreationLease {
    let mut lease = dom
        .begin_hosted_top_level_html(source, prepared, cancel)
        .unwrap();
    let DomHostedCreationStep::PermissionsPolicy(request) = lease.step(dom).unwrap() else {
        panic!("expected policy request");
    };
    let policy = DomInitialPermissionsPolicy::new(
        DomPermissionsPolicyValue::Enabled,
        DomPermissionsPolicyValue::Enabled,
    );
    let mut publication = Some(
        source
            .seal_permissions_policy(request, policy, cancel)
            .unwrap(),
    );
    lease
        .accept_permissions_policy(dom, source, &mut publication, cancel)
        .unwrap();
    lease
}

mod focus_removing_steps;

fn finish_local_html_destruction(
    dom: &mut Dom,
    source: &mut DomHostedSource,
    invocation: &mut DomHostedDestructionInvocation,
    cancel: &AtomicBool,
) {
    let mut destruction = invocation.cleanup(dom, source, cancel).unwrap();
    let DomHostedDestructionStep::EnqueueParentTask(request) = destruction.step(dom).unwrap()
    else {
        panic!("expected destruction enqueue");
    };
    let mut publication = Some(source.seal_destruction_queued(request, cancel).unwrap());
    destruction
        .accept_publication(dom, source, &mut publication, cancel)
        .unwrap();
    let DomHostedDestructionStep::Cleanup(request) = destruction.step(dom).unwrap() else {
        panic!("expected destruction cleanup");
    };
    let mut publication = Some(source.seal_top_level_cleanup(request, cancel).unwrap());
    destruction
        .accept_publication(dom, source, &mut publication, cancel)
        .unwrap();
    destruction.finish(dom, source, cancel).unwrap();
}

#[test]
fn local_html_top_level_publishes_a_loading_current_entry_with_an_empty_tree() {
    let cancel = cancel();
    let (mut dom, mut source, mut prepared) = prepared_top_level();
    let revision = dom.revision.0;
    let source_revision = source.revision().unwrap().0;
    let (mut lease, document, global, _) =
        publish_local_html_entry(&mut dom, &mut source, &mut prepared, false);
    assert_eq!(
        source.qualify_hosted_local_html_parser(&dom, document, global, &cancel),
        Err(DomError::SourceOperationPending.into())
    );
    let operation = lease.operation();
    let creation = lease.finish(&mut dom, &mut source, &cancel).unwrap();
    assert_eq!(dom.revision.0, revision + 4);
    assert_eq!(source.revision().unwrap().0, source_revision + 9);
    assert_eq!(dom.hosted_revisions, 0);

    let snapshot = dom.snapshot();
    let captured = snapshot.document(document).unwrap();
    assert_eq!(
        captured.metadata().creation_cause(),
        &DomDocumentCreationCause::HostedLocalHtml {
            source: source.id(),
            operation,
        }
    );
    assert_eq!(
        captured.metadata().url(),
        &DocumentString::from("about:blank")
    );
    assert_eq!(
        captured.metadata().mode(),
        surgeist_html::construction::HtmlDocumentMode::NoQuirks
    );
    assert!(!captured.metadata().allow_declarative_shadow_roots());
    assert!(
        snapshot
            .node(document.node())
            .unwrap()
            .children()
            .is_empty()
    );
    assert!(captured.is_fully_active().unwrap());
    assert_eq!(dom.global_document(global).unwrap(), document);
    let navigable = dom.hosted_navigable(creation.navigable()).unwrap();
    assert_eq!(navigable.current_entry(), creation.entry());
    assert_eq!(navigable.active_entry(), creation.entry());
    assert_eq!(
        dom.hosted_entry(creation.entry()).unwrap().document(),
        Some(document)
    );
    source
        .qualify_hosted_local_html_parser(&dom, document, global, &cancel)
        .unwrap();
}

#[test]
fn local_html_parser_qualification_rejects_foreign_mismatched_and_retired_authority() {
    let cancel = cancel();
    let (mut dom, mut source, mut prepared) = prepared_top_level();
    let (mut lease, document, global, _) =
        publish_local_html_entry(&mut dom, &mut source, &mut prepared, false);
    let creation = lease.finish(&mut dom, &mut source, &cancel).unwrap();
    let before_retirement = dom.snapshot();
    let (mut foreign_dom, foreign_source, _) = prepared_top_level();
    let mut batch = DomBatch::new();
    let foreign = batch.create_document();
    let foreign = foreign_dom
        .apply(batch, &cancel)
        .unwrap()
        .document(&foreign)
        .unwrap();
    assert_eq!(
        foreign_source.qualify_hosted_local_html_parser(&dom, document, global, &cancel),
        Err(DomHostedError::ForeignSource)
    );
    assert_eq!(
        source.qualify_hosted_local_html_parser(&dom, foreign, global, &cancel),
        Err(DomHostedError::StaleIdentity)
    );

    let mut invocation = dom
        .begin_hosted_top_level_destruction(&mut source, creation.navigable(), &cancel)
        .unwrap();
    assert_eq!(
        source.qualify_hosted_local_html_parser(&dom, document, global, &cancel),
        Err(DomHostedError::StaleIdentity)
    );
    finish_local_html_destruction(&mut dom, &mut source, &mut invocation, &cancel);
    assert_eq!(
        source.qualify_hosted_local_html_parser(&dom, document, global, &cancel),
        Err(DomHostedError::StaleIdentity)
    );
    assert!(
        before_retirement
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !dom.snapshot()
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

#[test]
fn hosted_top_level_scratch_admits_the_final_policy_operation_before_issuance() {
    let c = cancel();
    let (row_backing, issuer_scratch) = {
        let (dom, source, prepared) = prepared_top_level();
        let work = &prepared.as_ref().unwrap().preparation.work;
        let credit = source.retain(size_of::<Operation>(), &c).unwrap();
        let bank = dom
            .hosted_operations
            .prepare(
                source.next_operation_slot(work, &c).unwrap(),
                work,
                &source,
                0,
                None,
                operations::BodyCredit::Owned(credit),
                operations::Projection::default(),
                &c,
            )
            .unwrap();
        (
            bank.inspect_new_storage_bytes(),
            source.next_operation_reservation_scratch_bytes_for_fixture(),
        )
    };
    // PreparedRows owns empty paths inline and their raw allocations out of line.
    // Both the final Operation and the issuer reservation coexist with those
    // paths at distinct real stages; their larger stage sets the exact peak.
    let exact = size_of::<operations::PreparedRows>()
        + row_backing
        + size_of::<Operation>().max(issuer_scratch);
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
                assert_eq!(dom.hosted_operations.inspect_storage_bytes(), row_backing);
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
                    "exact Policy installation succeeded: operation_header={}, prepared_header={}, row_backing={row_backing}, issuer_scratch={issuer_scratch}, exact={exact}",
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
                "one-short final Policy payload admitted: operation={:?}, operation_header={}, prepared_header={}, row_backing={row_backing}, issuer_scratch={issuer_scratch}, exact={exact}",
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
    fixture_with(|_| {})
}

fn fixture_with(
    before_hosting: impl FnOnce(&mut Dom),
) -> (
    Dom,
    DomHostedSource,
    DomHostedCreationLease,
    Option<DomHostedPolicyPublication>,
) {
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
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    before_hosting(&mut dom);
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
        .seal_permissions_policy(
            request,
            DomInitialPermissionsPolicy::new(
                DomPermissionsPolicyValue::Enabled,
                DomPermissionsPolicyValue::Enabled,
            ),
            &c,
        )
        .unwrap();
    (dom, source, lease, Some(publication))
}

#[test]
fn incomplete_hosted_construction_refuses_before_consuming_observer_checkpoint() {
    let c = cancel();
    let (mut dom, mut source, mut lease, mut policy) = fixture_with(|dom| {
        let limits =
            crate::internal::child_list::DomChildListObservationLimits::new(1, 4, 4, 16).unwrap();
        dom.set_limits(dom.limits().with_child_list_observation(limits))
            .unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let document = dom.apply(setup, &c).unwrap().document(&document).unwrap();
        let observer = dom.create_child_list_observer(&c).unwrap();
        dom.observe_child_list(observer, document.node(), false, &c)
            .unwrap();
        let mut append = DomBatch::new();
        let comment = append.create_comment(document, "pending observer delivery");
        append.insert_before(&comment, document, None);
        dom.apply(append, &c).unwrap();
    });
    lease
        .accept_permissions_policy(&mut dom, &mut source, &mut policy, &c)
        .unwrap();
    let DomHostedCreationStep::NativeExecution(request) = lease.step(&dom).unwrap() else {
        panic!("actual native construction request");
    };
    let global = source.issue_global(request, &c).unwrap();
    let mut publication = Some(
        source
            .seal_native_execution(request, global, 100, None, &c)
            .unwrap(),
    );
    lease
        .accept_publication(&mut dom, &mut source, &mut publication, &c)
        .unwrap();
    let before = dom.snapshot();
    assert!(before.incomplete_hosted_operation().is_some());
    assert_eq!(dom.pending_native_tasks(), 1);
    let limits = crate::internal::select::DomNativeCheckpointLimits::new(
        1_000_000,
        1_000_000,
        crate::internal::slot::DomSlotQueryLimits::new(16, 16).unwrap(),
    )
    .unwrap();
    assert_eq!(
        dom.native_checkpoint_step(limits, &c).unwrap_err(),
        DomError::SourceOperationPending
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.pending_native_tasks(), 1);
    assert!(dom.has_pending_child_lists());
    assert!(dom.child_list_turn.is_none());
}
fn used(source: &DomHostedSource) -> usize {
    source.limits().retained_bytes() - source.remaining_bytes().unwrap()
}

#[test]
fn complete_policy_receipt_preserves_independent_values_and_atomic_constructor_custody() {
    use DomPermissionsPolicyValue::{Disabled, Enabled};
    // This tests the original trusted policy-publication/constructor boundary,
    // not Runtime's policy evaluator or a completed Window/focusing service.
    for (fullscreen, focus) in [
        (Enabled, Enabled),
        (Enabled, Disabled),
        (Disabled, Enabled),
        (Disabled, Disabled),
    ] {
        let (mut dom, mut source, mut lease, _) = fixture();
        let policy = DomInitialPermissionsPolicy::new(fullscreen, focus);
        assert_eq!(policy.inherited_fullscreen(), fullscreen);
        assert_eq!(policy.inherited_focus_without_user_activation(), focus);
        assert_eq!(policy.profile(), DomInitialPermissionsPolicyProfile::FullscreenAndFocusWithoutUserActivationSelfOpaqueInitial);
        let DomHostedCreationStep::PermissionsPolicy(request) = lease.step(&dom).unwrap() else {
            panic!("original Policy phase");
        };
        assert_eq!(request.profile(), policy.profile());
        let mut publication = Some(
            source
                .seal_permissions_policy(request, policy, &cancel())
                .unwrap(),
        );
        let stamp = dom.snapshot().stamp();
        let revision = source.revision().unwrap();
        assert_eq!(dom.hosted_operation(lease.operation).unwrap().policy, None);
        assert_eq!(
            lease.accept_permissions_policy(
                &mut dom,
                &mut source,
                &mut publication,
                &AtomicBool::new(true)
            ),
            Err(DomHostedError::Dom(DomError::Cancelled))
        );
        assert_eq!(publication.as_ref().unwrap().policy, policy);
        assert_eq!(dom.hosted_operation(lease.operation).unwrap().policy, None);
        assert_eq!(source.revision().unwrap(), revision);
        assert_eq!(dom.snapshot().stamp(), stamp);
        lease
            .accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel())
            .unwrap();
        assert!(publication.is_none());
        assert_eq!(
            dom.hosted_operation(lease.operation).unwrap().policy,
            Some(policy)
        );
        assert_eq!(
            lease.accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel()),
            Err(DomHostedError::WrongPhase)
        );
        let DomHostedCreationStep::NativeExecution(request) = lease.step(&dom).unwrap() else {
            panic!("actual constructor phase");
        };
        let global = source.issue_global(request, &cancel()).unwrap();
        let mut execution = Some(
            source
                .seal_native_execution(request, global, 100, None, &cancel())
                .unwrap(),
        );
        lease
            .accept_publication(&mut dom, &mut source, &mut execution, &cancel())
            .unwrap();
        let DomHostedCreationStep::MakeActive(request) = lease.step(&dom).unwrap() else {
            panic!("constructed, not yet active Document");
        };
        let document = request.document();
        assert_eq!(
            dom.snapshot()
                .document(document)
                .unwrap()
                .focused_area_designation(),
            crate::internal::document::DomFocusedAreaDesignation::KnownNull
        );
        assert_eq!(
            dom.snapshot()
                .document(document)
                .unwrap()
                .metadata()
                .initial_permissions_policy(),
            Some(policy)
        );
        let staged = Draft::new(&dom, Arc::new(BatchIdentity), &cancel()).unwrap();
        assert_eq!(
            staged.document_fully_active(staged.resolve(document.into()).unwrap(), &cancel()),
            Err(DomError::HostedSourceUnavailable)
        );
    }
}

#[test]
fn foreign_complete_policy_receipt_cannot_partially_install_either_feature() {
    let (mut dom, mut source, mut lease, mut original) = fixture();
    let (_other_dom, _other_source, _other_lease, mut foreign) = fixture();
    let stamp = dom.snapshot().stamp();
    let revision = source.revision().unwrap();
    assert_eq!(
        lease.accept_permissions_policy(&mut dom, &mut source, &mut foreign, &cancel()),
        Err(DomHostedError::WrongPhase)
    );
    assert!(foreign.is_some());
    assert!(original.is_some());
    assert_eq!(dom.hosted_operation(lease.operation).unwrap().policy, None);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(source.revision().unwrap(), revision);
    lease
        .accept_permissions_policy(&mut dom, &mut source, &mut original, &cancel())
        .unwrap();
    assert!(original.is_none());
    assert_eq!(
        dom.hosted_operation(lease.operation).unwrap().policy,
        Some(DomInitialPermissionsPolicy::new(
            DomPermissionsPolicyValue::Enabled,
            DomPermissionsPolicyValue::Enabled
        ))
    );
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
            .seal_native_execution(request, global, 100, None, &c)
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
        surgeist_html::security::opener_policy::HtmlOpenerPolicyValue::UnsafeNone
    );
    assert_eq!(
        request.opener_policy().report_only_value(),
        surgeist_html::security::opener_policy::HtmlOpenerPolicyValue::UnsafeNone
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
        matches!(HostedDocumentInputs::form(&source,&work,&origin,HostedDocumentProfile::InitialAboutBlank,&cancel()),
        Err(DomHostedError::Limit { resource: DomHostedResource::ScratchBytes,limit }) if limit==scratch-1)
    );
    assert_eq!(used(&source), base);
    source
        .set_limits(limits(8_000_000, scratch, 1_000_000), &cancel())
        .unwrap();
    let before = work.spent.load(Ordering::Relaxed);
    let inputs = HostedDocumentInputs::form(
        &source,
        &work,
        &origin,
        HostedDocumentProfile::InitialAboutBlank,
        &cancel(),
    )
    .unwrap();
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
            matches!(HostedDocumentInputs::form(&source,&work,&origin,HostedDocumentProfile::InitialAboutBlank,&cancel()),Err(DomHostedError::WorkLimit { limit }) if limit==before+allowance)
        );
        assert_eq!(used(&source), base);
        assert_eq!(work.spent.load(Ordering::Relaxed), before + allowance);
    }
}

#[test]
fn native_select_toggle_publishes_with_its_original_notification() {
    use crate::internal::{
        source::{
            DomSourceAdvance, DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest,
        },
        store::source::operation::element_queue::{
            DomQueuedSourceElementTask, DomSourceElementTaskInfo,
        },
    };
    use surgeist_html::{
        bounded::select::{HtmlSelectNotificationAction as Action, HtmlSelectNotificationEvent},
        values::Namespace,
    };

    let cancel = cancel();
    let (mut dom, mut hosted, mut prepared) = prepared_top_level();
    let (mut lease, document, _, _) =
        publish_local_html_entry(&mut dom, &mut hosted, &mut prepared, false);
    lease.finish(&mut dom, &mut hosted, &cancel).unwrap();
    let mut domain = dom
        .limits()
        .with_select_processing(16, 1_000_000, 1_000_000)
        .unwrap();
    domain.slot_references = 4;
    dom.set_limits(domain).unwrap();
    let source = dom.bind_element_task_source(&cancel).unwrap();
    let mut setup = DomBatch::new();
    let select = setup.create_element_ns(document, Namespace::html(), "select");
    let option = setup.create_element_ns(document, Namespace::html(), "option");
    setup.set_attribute(&select, "multiple", "");
    setup.set_attribute(&option, "label", "before the task");
    setup.insert_before(&option, &select, None);
    setup.insert_before(&select, document, None);
    let setup = dom.apply(setup, &cancel).unwrap();
    let select = setup.created(&select).unwrap().unwrap();
    let option = setup.created(&option).unwrap().unwrap();
    let before = dom.snapshot();
    let flags = |snapshot: &DomSnapshot| {
        snapshot
            .node(option)
            .unwrap()
            .as_option()
            .unwrap()
            .state()
            .flags()
    };
    let validity = |snapshot: &DomSnapshot| {
        snapshot
            .node(select)
            .unwrap()
            .as_select()
            .unwrap()
            .state()
            .flags()
            .user_validity()
    };
    let limits = DomSourceLimits::new(64_000_000, 2_000_000, 10_000_000, 64, 1024, 1024, 4);
    let edit = DomSourceEdit::toggle_select_option_for_user(select, option, before.stamp());
    let mut lease = dom.begin_source_edit(edit, limits, &cancel).unwrap();
    let DomSourceAdvance::Progress(DomSourceProgress::Request(DomSourceRequest::ElementTaskQueue(
        request,
    ))) = lease.advance_for_runtime(&cancel).unwrap()
    else {
        panic!("accepted user toggle reaches its original handoff")
    };
    {
        let prepared = lease
            .prepare_element_task_queue(request, &source, 10, &cancel)
            .unwrap();
        let mut prospective = prepared.tasks();
        let Some(DomSourceElementTaskInfo::SelectNotification(info)) = prospective.next() else {
            panic!("toggle has one notification")
        };
        assert_eq!(info.id().target(), select);
        assert_eq!(info.queue_association().document(), document);
        assert!(prospective.next().is_none());
    }
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(!flags(&lease.snapshot()).selectedness());
    let (_, mut queued) = lease
        .prepare_element_task_queue(request, &source, 10, &cancel)
        .unwrap()
        .publish()
        .unwrap();
    let Some(DomQueuedSourceElementTask::SelectNotification(mut task)) = queued.next() else {
        panic!("original task transferred with selectedness")
    };
    assert!(queued.next().is_none());
    drop(queued);
    drop(lease);
    assert!(flags(&dom.snapshot()).selectedness());
    assert!(flags(&dom.snapshot()).dirtiness());
    assert!(!validity(&dom.snapshot()));
    assert!(!flags(&before).selectedness());

    let mut update = DomBatch::new();
    update.set_attribute(option, "label", "after the toggle");
    dom.apply(update, &cancel).unwrap();
    let staged = dom.snapshot();
    assert!(matches!(
        dom.apply_select_notification_local_action(&mut task, limits, &mut 0, &cancel),
        Err(DomError::SourceWorkLimit { .. })
    ));
    assert_eq!(dom.snapshot().stamp(), staged.stamp());
    assert_eq!(task.action(), Some(Action::MarkUserValidity));
    let mut work = 10_000_000;
    dom.apply_select_notification_local_action(&mut task, limits, &mut work, &cancel)
        .unwrap();
    assert!(validity(&dom.snapshot()));
    assert_eq!(task.action(), Some(Action::UpdateSelectedcontent));
    let mut content = source
        .select_notification_content(&mut dom, &mut task, limits, &cancel)
        .unwrap();
    content.request_cancel();
    drop(content);
    assert!(dom.pending_source_edit().is_some());
    let mut content = source
        .select_notification_content(&mut dom, &mut task, limits, &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = content.advance(&cancel).unwrap() else {
        panic!("multiple Selectedcontent operation returns as specified")
    };
    task.accept_content_completion(&done).unwrap();
    assert!(task.accept_content_completion(&done).is_err());
    drop(content);
    dom.apply_select_notification_local_action(&mut task, limits, &mut work, &cancel)
        .unwrap();
    let after = dom.snapshot();
    let root = after.node(select).unwrap().hosted_shadow_root().unwrap();
    let slot = after.node(root).unwrap().children()[0];
    let fallback = after.node(slot).unwrap().children()[0];
    let label = after.node(fallback).unwrap().children()[0];
    assert!(matches!(
        after.node(label).unwrap().kind(),
        crate::internal::snapshot::DomNodeKind::Text(value) if value == &DocumentString::from("after the toggle")
    ));
    assert_eq!(
        task.action(),
        Some(Action::Dispatch(HtmlSelectNotificationEvent::Input))
    );
    assert!(dom.pending_source_edit().is_none());

    // Dropping an admitted notification's cursor leaves the original task
    // owner able to settle its already opened canonical Source work.
    let edit = DomSourceEdit::toggle_select_option_for_user(select, option, after.stamp());
    let mut lease = dom.begin_source_edit(edit, limits, &cancel).unwrap();
    let DomSourceAdvance::Progress(DomSourceProgress::Request(DomSourceRequest::ElementTaskQueue(
        request,
    ))) = lease.advance_for_runtime(&cancel).unwrap()
    else {
        panic!("second toggle reaches its own queue admission")
    };
    let (_, mut queued) = lease
        .prepare_element_task_queue(request, &source, 11, &cancel)
        .unwrap()
        .publish()
        .unwrap();
    let Some(DomQueuedSourceElementTask::SelectNotification(mut abandoned)) = queued.next() else {
        panic!("second toggle transfers one original task")
    };
    drop(queued);
    drop(lease);
    dom.apply_select_notification_local_action(&mut abandoned, limits, &mut work, &cancel)
        .unwrap();
    let id = abandoned.info().id();
    let lease = source
        .select_notification_content(&mut dom, &mut abandoned, limits, &cancel)
        .unwrap();
    let operation = lease.operation();
    drop(lease);
    drop(abandoned);
    assert!(matches!(
        dom.resume_source_edit(operation, limits, &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    let (original, mut recovery) = source
        .resume_select_notification_content(&mut dom, operation, limits, &cancel)
        .unwrap();
    assert_eq!(original, id);
    let DomSourceProgress::Complete(settled) = recovery.advance(&cancel).unwrap() else {
        panic!("exact owner drains the outstanding multiple Select Source operation")
    };
    assert_eq!(settled.operation(), operation);
    drop(recovery);
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn hosted_embed_setup_refuses_attribute_and_tree_publication() {
    use crate::internal::{
        shadow::DomShadowRootInit,
        source::{DomSourceEdit, DomSourceLimits},
    };
    use surgeist_html::{construction::HtmlShadowRootMode, values::Namespace};

    let cancel = cancel();
    let (mut dom, mut source, mut prepared) = prepared_top_level();
    let (mut lease, document, _, _) =
        publish_local_html_entry(&mut dom, &mut source, &mut prepared, false);
    lease.finish(&mut dom, &mut source, &cancel).unwrap();
    let limits = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);

    let mut creation = DomBatch::new();
    let root = creation.create_element_ns(document, Namespace::html(), "div");
    let child = creation.create_element_ns(document, Namespace::html(), "embed");
    let detached = creation.create_element_ns(document, Namespace::html(), "embed");
    let host = creation.create_element_ns(document, Namespace::html(), "div");
    let shadow = creation.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let shadow_embed = creation.create_element_ns(document, Namespace::html(), "embed");
    creation.set_attribute(&child, "src", "");
    creation.set_attribute(&child, "type", "image/svg+xml");
    creation.set_attribute(&detached, "src", "relative.svg");
    creation.insert_before(&root, document, None);
    creation.insert_before(&child, &root, None);
    creation.insert_before(&host, &root, None);
    creation.insert_before(&shadow_embed, &shadow, None);
    creation.set_attribute(&shadow_embed, "src", "relative.svg");
    let commit = dom.apply(creation, &cancel).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();
    let shadow_embed = commit.created(&shadow_embed).unwrap().unwrap();
    let before = dom.snapshot();
    assert!(
        before
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let usage = dom.usage();

    let edit = DomSourceEdit::set_native_attribute(child, "src", "relative.svg");
    let refusal = match dom.begin_source_edit(edit, limits, &cancel) {
        Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
        Err(error) => error,
    };
    assert_eq!(refusal, DomError::EmbedSetupUnavailable);
    let mut insertion = DomBatch::new();
    insertion.set_attribute(root, "id", "private prefix");
    insertion.insert_before(detached, root, None);
    assert_eq!(
        dom.apply(insertion, &cancel).unwrap_err(),
        DomError::EmbedSetupUnavailable
    );
    let current = dom.snapshot();
    assert_eq!(current.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        current
            .node(root)
            .unwrap()
            .attribute(&"id".into())
            .is_none()
    );
    assert!(current.node(detached).unwrap().parent().is_none());
    let child = current.node(child).unwrap();
    assert_eq!(child.attribute(&"src".into()), Some(&"".into()));
    assert!(child.as_element().unwrap().is_inactive_embed());
    let shadow_embed = current.node(shadow_embed).unwrap();
    assert!(shadow_embed.is_connected());
    assert!(shadow_embed.as_element().unwrap().is_inactive_embed());
    assert_eq!(
        shadow_embed.attribute(&"src".into()),
        Some(&"relative.svg".into())
    );

    let mut same_position = DomBatch::new();
    same_position.insert_before(child.id(), root, Some(host.into()));
    dom.apply(same_position, &cancel).unwrap();
    assert_eq!(dom.snapshot().stamp(), before.stamp());

    let mut removal = DomBatch::new();
    removal.detach(child.id());
    dom.apply(removal, &cancel).unwrap();
    let before = dom.snapshot();
    assert!(!before.node(child.id()).unwrap().is_connected());
    let usage = dom.usage();
    let mut later = DomBatch::new();
    later.set_attribute(child.id(), "src", "late.svg");
    assert_eq!(
        dom.apply(later, &cancel).unwrap_err(),
        DomError::EmbedSetupUnavailable
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);

    let mut transient = DomBatch::new();
    let fresh = transient.create_element_ns(document, Namespace::html(), "embed");
    transient.insert_before(&fresh, root, None);
    transient.detach(&fresh);
    transient.set_attribute(&fresh, "src", "relative.svg");
    let commit = dom.apply(transient, &cancel).unwrap();
    let fresh = commit.created(&fresh).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let node = snapshot.node(fresh).unwrap();
    assert!(!node.is_connected());
    assert!(node.as_element().unwrap().is_inactive_embed());
    assert_eq!(node.attribute(&"src".into()), Some(&"relative.svg".into()));
    let mut update = DomBatch::new();
    update.set_attribute(fresh, "src", "second.svg");
    dom.apply(update, &cancel).unwrap();

    let mut create = DomBatch::new();
    let idle = create.create_element_ns(document, Namespace::html(), "embed");
    let idle = dom
        .apply(create, &cancel)
        .unwrap()
        .created(&idle)
        .unwrap()
        .unwrap();
    let before = dom.snapshot();
    let mut transient = DomBatch::new();
    transient.insert_before(idle, root, None);
    transient.detach(idle);
    dom.apply(transient, &cancel).unwrap();
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}
