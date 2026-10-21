//! Real direct-DOM composition. Trusted publications model the host's existing
//! native protocol; no Runtime Window/task bank or event receipt is fabricated.
use super::*;
use crate::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest};
use surgeist_css::{CssParserContext, CssUtf16InputLimits};
use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 50_000_000, 64, 4096, 4096, 1)
}
fn constructor() -> (Dom, DomHostedSource, DocumentId) {
    let mut dom = Dom::new(
        DomLimits::new(
            2048,
            1_000_000,
            4096,
            128,
            128,
            32,
            32_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut source = dom
        .register_hosted_source(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                64_000_000,
                64_000_000,
                50_000_000,
            ),
            &cancel(),
        )
        .unwrap();
    let host = source.issue_host(&cancel()).unwrap();
    let mut prefix = Some(source.prepare_top_level(host, &cancel()).unwrap());
    let mut input = Some(source.seal_top_level(&mut prefix, &cancel()).unwrap());
    let mut lease = dom
        .begin_hosted_top_level(&mut source, &mut input, &cancel())
        .unwrap();
    loop {
        let publication = match lease.step(&dom).unwrap() {
            DomHostedCreationStep::PermissionsPolicy(request) => {
                let mut publication = Some(
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
                    .accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel())
                    .unwrap();
                continue;
            }
            DomHostedCreationStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel()).unwrap();
                source
                    .seal_native_execution(request, global, 0, &cancel())
                    .unwrap()
            }
            DomHostedCreationStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel()).unwrap()
            }
            DomHostedCreationStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 100, &cancel()).unwrap()
            }
            DomHostedCreationStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel()).unwrap();
                let nav = source.issue_navigable(request, &cancel()).unwrap();
                let uuid = |last| {
                    let mut value = [0; 36];
                    for (out, unit) in value
                        .iter_mut()
                        .zip("00000000-0000-4000-8000-000000000000".encode_utf16())
                    {
                        *out = unit;
                    }
                    value[35] = last;
                    value
                };
                let mut identity = Some(DomHostedEntryInitialization::new(uuid(49), uuid(50)));
                source
                    .seal_initial_entry(request, entry, nav, &mut identity, &cancel())
                    .unwrap()
            }
            DomHostedCreationStep::Complete(_) => break,
        };
        lease
            .accept_publication(&mut dom, &mut source, &mut Some(publication), &cancel())
            .unwrap();
    }
    let document = lease
        .finish(&mut dom, &mut source, &cancel())
        .unwrap()
        .document();
    (dom, source, document)
}

fn style_resource(
    dom: &mut Dom,
    document: DocumentId,
    parent: NodeId,
    root_node: NodeId,
    remove: bool,
) -> (
    DomStyleCssomRoot,
    surgeist_cssom::CssomSheetId,
    CssomSnapshot,
) {
    let mut setup = DomBatch::new();
    let style = setup.create_element_ns(document, crate::Namespace::html(), "style");
    let text = setup.create_text(document, "a { color: red }");
    setup.insert_before(&text, &style, None);
    let style = dom
        .apply(setup, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let root = DomStyleCssomRoot::new(
        root_node,
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, parent, None),
            source_limits(),
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("actual policy request")
    };
    lease
        .accept_inline_csp(
            &policy,
            &crate::DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("actual create request")
    };
    lease.bind_cssom_root(&create, &root, &cancel()).unwrap();
    let create = lease
        .qualify_cssom(&create, root.root(), &root.snapshot().unwrap())
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "direct-style".into(),
        revision: 1,
    };
    lease
        .bind_cssom_owner(
            &create,
            crate::DomStyleCssomOwnerBinding::new(&create, version.clone(), "direct-style".into())
                .unwrap(),
            &cancel(),
        )
        .unwrap();
    let parser = CssParserContext::default();
    let body = parser
        .parse_sheet_utf16_with_limits(
            create.source().text().clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            Arc::new(Vec::new()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut inputs = CssomSheetInputs::external(version, true);
    inputs.owner = Some("direct-style".into());
    let mut batch = root
        .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
        .unwrap();
    let construction = batch
        .create_parsed_sheet_with_media_reports(body, media, inputs)
        .unwrap();
    let commit = root.commit(&mut Some(batch)).unwrap();
    let sheet = commit.sheet(construction.ticket()).unwrap().clone();
    let mut notification = lease
        .accept_cssom_publication(
            &create,
            commit,
            root.snapshot().unwrap(),
            Some(construction.ticket()),
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let historical = root.snapshot().unwrap();
    if remove {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::detach(style), source_limits(), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual removal request")
        };
        let request = lease
            .qualify_cssom(&request, root.root(), &root.snapshot().unwrap())
            .unwrap();
        let mut batch = root
            .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
            .unwrap();
        batch.remove_sheet(&sheet).unwrap();
        let commit = root.commit(&mut Some(batch)).unwrap();
        assert!(
            lease
                .accept_cssom_publication(&request, commit, root.snapshot().unwrap(), None)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    // This direct host has no dispatched Rust callbacks. Its genuine native
    // notification permission drives the real canonical post-event cleanup;
    // it does not forge a Runtime Delivered receipt or delete a task bank.
    assert!(matches!(
        dom.finish_style_notification(
            &mut notification,
            crate::DomStyleNotificationDelivery::Native,
            1,
            &cancel(),
        )
        .unwrap(),
        crate::DomStyleNotificationCompletion::Complete(_)
    ));
    (root, sheet, historical)
}

struct Fixture {
    dom: Dom,
    source: DomHostedSource,
    document: DocumentId,
    root: DomStyleCssomRoot,
    sheet: surgeist_cssom::CssomSheetId,
    historical: CssomSnapshot,
    invocation: DomHostedDestructionInvocation,
    lease: DomHostedDestructionLease,
    cleanup: Option<DomHostedDestructionPublication>,
    other_root: Option<DomStyleCssomRoot>,
}
impl Fixture {
    fn new(remove: bool, unrelated: usize) -> Self {
        Self::with_roots(remove, unrelated, false, None)
    }
    fn with_roots(
        remove: bool,
        unrelated: usize,
        selected_shadow: bool,
        other_document: Option<bool>,
    ) -> Self {
        let (mut dom, mut source, document) = constructor();
        let mut batch = DomBatch::new();
        // A genuine detached participant belongs to the selected Document group.
        batch.create_element_ns(document, crate::Namespace::html(), "style");
        for _ in 0..unrelated {
            let other = batch.create_document();
            batch.create_element_ns(&other, crate::Namespace::html(), "style");
        }
        dom.apply(batch, &cancel()).unwrap();
        let document_element = dom
            .snapshot()
            .document(document)
            .unwrap()
            .document_element()
            .unwrap();
        let shadow = |dom: &mut Dom| {
            let mut setup = DomBatch::new();
            let host = setup.create_element_ns(document, crate::Namespace::html(), "div");
            setup.insert_before(&host, document_element, None);
            let shadow = setup.attach_shadow(
                &host,
                surgeist_html::HtmlShadowRootProperties::new(
                    surgeist_html::HtmlShadowRootMode::Open,
                    surgeist_html::HtmlSlotAssignment::Manual,
                    false,
                    true,
                    false,
                ),
            );
            dom.apply(setup, &cancel())
                .unwrap()
                .created(&shadow)
                .unwrap()
                .unwrap()
        };
        let root_node = if selected_shadow {
            shadow(&mut dom)
        } else {
            document.node()
        };
        let parent = if selected_shadow {
            root_node
        } else {
            document_element
        };
        let (root, sheet, historical) =
            style_resource(&mut dom, document, parent, root_node, remove);
        let other_root = other_document.map(|different| {
            if different {
                let mut setup = DomBatch::new();
                let other = setup.create_document();
                let other = dom
                    .apply(setup, &cancel())
                    .unwrap()
                    .document(&other)
                    .unwrap();
                style_resource(&mut dom, other, other.node(), other.node(), true).0
            } else {
                let shadow = shadow(&mut dom);
                style_resource(&mut dom, document, shadow, shadow, true).0
            }
        });
        let nav = dom
            .snapshot()
            .document(document)
            .unwrap()
            .navigable()
            .unwrap();
        let mut invocation = dom
            .begin_hosted_top_level_destruction(&mut source, nav, &cancel())
            .unwrap();
        let mut lease = invocation
            .cleanup(&mut dom, &mut source, &cancel())
            .unwrap();
        let DomHostedDestructionStep::EnqueueParentTask(request) = lease.step(&dom).unwrap() else {
            panic!("actual enqueue request")
        };
        let mut queued = Some(source.seal_destruction_queued(request, &cancel()).unwrap());
        lease
            .accept_publication(&mut dom, &mut source, &mut queued, &cancel())
            .unwrap();
        let DomHostedDestructionStep::Cleanup(request) = lease.step(&dom).unwrap() else {
            panic!("actual cleanup request")
        };
        let mut cleanup = Some(source.seal_top_level_cleanup(request, &cancel()).unwrap());
        assert_eq!(
            lease.accept_publication(&mut dom, &mut source, &mut cleanup, &cancel()),
            Err(DomHostedError::PendingDocumentObligations)
        );
        Self {
            dom,
            source,
            document,
            root,
            sheet,
            historical,
            invocation,
            lease,
            cleanup,
            other_root,
        }
    }
    fn prepare(
        &mut self,
        c: &AtomicBool,
    ) -> Result<DomHostedStyleRootWithdrawalPreparation<'_, '_>, DomHostedError> {
        self.lease.prepare_style_root_withdrawal(
            &mut self.dom,
            &self.source,
            self.cleanup.as_ref().unwrap(),
            &self.root,
            c,
        )
    }
    fn work(&self) -> Arc<Work> {
        self.dom
            .hosted_operation(self.lease.operation)
            .unwrap()
            .request()
            .work
            .clone()
    }
}

#[test]
fn actual_bank_and_producer_fence_then_same_cleanup_preserve_tree_and_history() {
    let mut f = Fixture::new(true, 0);
    let stamp = f.dom.snapshot().stamp();
    let revision = f.root.snapshot().unwrap().revision().clone();
    let source_revision = f.source.revision().unwrap();
    let alias = f.root.duplicate();
    let prepared = f.prepare(&cancel()).unwrap();
    assert!(prepared.withdrawal_snapshot().sheets().is_empty());
    assert!(matches!(
        alias.snapshot(),
        Err(crate::DomStyleCssomRootError::Busy)
    ));
    drop(prepared);
    assert!(
        f.dom
            .style_cssom_root(f.root.root())
            .unwrap()
            .unwrap()
            .same_owner(&f.root)
    );
    let retained_before = f.source.retained_storage_bytes();
    let prepared = f.prepare(&cancel()).unwrap();
    let published = prepared.publish();
    assert!(published._dom.style_cssom_roots.get(alias.root()).is_none());
    assert!(published.retired.get(alias.root()).is_some());
    assert!(matches!(
        alias.snapshot(),
        Err(crate::DomStyleCssomRootError::Busy)
    ));
    let outcome = published.finish();
    assert_eq!(
        f.source.retained_storage_bytes(),
        retained_before + size_of::<DomHostedStyleRootWithdrawal>()
    );
    assert_eq!(outcome.root(), f.document.node());
    assert_eq!(outcome.document(), f.document);
    assert_eq!(outcome.cleanup_operation(), f.lease.operation());
    assert_eq!(outcome.stamp(), stamp);
    assert_eq!(outcome.withdrawal_cssom_snapshot().revision(), &revision);
    assert_eq!(f.dom.snapshot().stamp(), stamp);
    assert_eq!(f.source.revision().unwrap(), source_revision);
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::Dom(DomError::StyleReceiptMismatch)
    );
    f.lease
        .accept_publication(&mut f.dom, &mut f.source, &mut f.cleanup, &cancel())
        .unwrap();
    f.lease
        .finish(&mut f.dom, &mut f.source, &cancel())
        .unwrap();
    assert!(
        !f.dom
            .snapshot()
            .document(f.document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(f.dom.snapshot().node(f.document.node()).is_ok());
    assert!(
        f.historical
            .sheet(&f.sheet)
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert!(
        outcome
            .withdrawal_cssom_snapshot()
            .sheet(&f.sheet)
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    let mut batch = alias
        .batch(CssomEditGuard::from_snapshot(&alias.snapshot().unwrap()))
        .unwrap();
    batch.set_disabled(&f.sheet, true).unwrap();
    alias.commit(&mut Some(batch)).unwrap();
    assert_ne!(
        alias.snapshot().unwrap().revision(),
        outcome.withdrawal_cssom_snapshot().revision()
    );
    assert!(
        !outcome
            .withdrawal_cssom_snapshot()
            .sheet(&f.sheet)
            .unwrap()
            .inputs()
            .disabled
    );
    assert_eq!(
        f.invocation.request(&f.dom).unwrap().cleanup_operation(),
        f.lease.operation()
    );
}

#[test]
fn actual_busy_poisoned_foreign_and_equal_revision_other_owner_keep_registration() {
    let mut f = Fixture::new(true, 0);
    let alias = f.root.duplicate();
    let held = alias.lock().unwrap();
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::Dom(DomError::StyleCssomRootBusy)
    );
    drop(held);
    let other = DomStyleCssomRoot::new(
        f.root.root(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    // Independent genuine create/remove publications reach the same numeric
    // revision; that does not grant the canonical producer's owner identity.
    let parser = CssParserContext::default();
    let body = parser
        .parse_sheet_utf16_with_limits(
            Arc::new("b {}".encode_utf16().collect()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            Arc::new(Vec::new()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut batch = other
        .batch(CssomEditGuard::from_snapshot(&other.snapshot().unwrap()))
        .unwrap();
    let created = batch
        .create_parsed_sheet_with_media_reports(
            body,
            media,
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "independent".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let commit = other.commit(&mut Some(batch)).unwrap();
    let sheet = commit.sheet(created.ticket()).unwrap().clone();
    let mut batch = other
        .batch(CssomEditGuard::from_snapshot(&other.snapshot().unwrap()))
        .unwrap();
    batch.remove_sheet(&sheet).unwrap();
    other.commit(&mut Some(batch)).unwrap();
    assert_eq!(
        other.snapshot().unwrap().revision().value(),
        f.root.snapshot().unwrap().revision().value()
    );
    assert_ne!(
        other.snapshot().unwrap().revision().owner(),
        f.root.snapshot().unwrap().revision().owner()
    );
    assert_eq!(
        f.lease
            .prepare_style_root_withdrawal(
                &mut f.dom,
                &f.source,
                f.cleanup.as_ref().unwrap(),
                &other,
                &cancel()
            )
            .unwrap_err(),
        DomHostedError::Dom(DomError::StyleReceiptMismatch)
    );
    let foreign = Fixture::new(true, 0);
    assert_eq!(
        f.lease
            .prepare_style_root_withdrawal(
                &mut f.dom,
                &foreign.source,
                f.cleanup.as_ref().unwrap(),
                &f.root,
                &cancel()
            )
            .unwrap_err(),
        DomHostedError::ForeignSource
    );
    assert_eq!(
        f.lease
            .prepare_style_root_withdrawal(
                &mut f.dom,
                &f.source,
                foreign.cleanup.as_ref().unwrap(),
                &f.root,
                &cancel()
            )
            .unwrap_err(),
        DomHostedError::WrongPhase
    );
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = alias.lock().unwrap();
        panic!("poison the actual selected producer");
    }));
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::Dom(DomError::StyleCssomRootPoisoned)
    );
    assert!(
        f.dom
            .style_cssom_root(f.root.root())
            .unwrap()
            .unwrap()
            .same_owner(&f.root)
    );
    assert!(f.root.poisoned_snapshot_for_test().sheets().is_empty());
    assert!(f.cleanup.is_some());
}

#[test]
fn real_nonempty_and_externally_removed_sheet_do_not_hide_current_canonical_association() {
    let mut f = Fixture::new(false, 0);
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::PendingDocumentObligations
    );
    let mut batch = f
        .root
        .batch(CssomEditGuard::from_snapshot(&f.root.snapshot().unwrap()))
        .unwrap();
    batch.remove_sheet(&f.sheet).unwrap();
    f.root.commit(&mut Some(batch)).unwrap();
    assert!(f.root.snapshot().unwrap().sheets().is_empty());
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::Dom(DomError::StyleProcessingUnavailable)
    );
    assert!(f.dom.has_style_cssom_roots());
    assert!(f.cleanup.is_some());
}

#[test]
fn cancelled_and_finite_storage_refusal_keep_same_cleanup_for_raised_retry() {
    let mut f = Fixture::new(true, 0);
    let work = f.work();
    let before = work.spent.load(Ordering::Relaxed);
    let revision = f.source.revision().unwrap();
    assert_eq!(
        f.prepare(&AtomicBool::new(true)).unwrap_err(),
        DomHostedError::Dom(DomError::Cancelled)
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), before);
    let limits = f.source.limits();
    let used = f.source.retained_storage_bytes();
    let fixed = size_of::<DomHostedStyleRootWithdrawalPreparation<'_, '_>>();
    f.source
        .set_limits(
            DomHostedLimits::new(
                limits.records(),
                used + fixed - 1,
                limits.scratch_bytes(),
                limits.work(),
            ),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        f.prepare(&cancel()),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            ..
        })
    ));
    assert_eq!(f.source.retained_storage_bytes(), used);
    assert!(f.dom.has_style_cssom_roots());
    assert_eq!(f.source.revision().unwrap(), revision);
    assert!(Arc::ptr_eq(&work, &f.work()));
    f.source.set_limits(limits, &cancel()).unwrap();
    let begin = work.spent.load(Ordering::Relaxed);
    drop(f.prepare(&cancel()).unwrap());
    let cost = work.spent.load(Ordering::Relaxed) - begin;
    let spent = work.spent.load(Ordering::Relaxed);
    f.source
        .set_limits(
            DomHostedLimits::new(
                limits.records(),
                limits.retained_bytes(),
                limits.scratch_bytes(),
                spent + cost - 1,
            ),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        f.prepare(&cancel()),
        Err(DomHostedError::WorkLimit { .. })
            | Err(DomHostedError::Dom(DomError::SourceWorkLimit { .. }))
    ));
    assert!(f.dom.has_style_cssom_roots());
    assert!(work.spent.load(Ordering::Relaxed) > spent);
    f.source.set_limits(limits, &cancel()).unwrap();
    let outcome = f.prepare(&cancel()).unwrap().publish().finish();
    assert_eq!(outcome.cleanup_operation(), f.lease.operation());
    assert!(f.cleanup.is_some());
    f.lease
        .accept_publication(&mut f.dom, &mut f.source, &mut f.cleanup, &cancel())
        .unwrap();
    assert!(f.lease.destruction_proof().is_some());
}

#[test]
fn maintained_document_discovery_visits_only_qualified_members_on_original_work() {
    let mut costs = Vec::new();
    for unrelated in [0, 128] {
        let mut f = Fixture::new(true, unrelated);
        let work = f.work();
        let operation = f.dom.hosted_operation(f.lease.operation).unwrap();
        let draft = f.dom.hosted_draft(operation, &f.source, &cancel()).unwrap();
        let probe_before = work.spent.load(Ordering::Relaxed);
        let mut members = 0;
        f.dom
            .state
            .inbound
            .visit_target(
                Key::Existing(f.document.node()),
                |owner| {
                    draft.source_step(&cancel())?;
                    let Key::Existing(id) = owner else {
                        panic!("canonical member")
                    };
                    assert_eq!(f.dom.state.node(f.dom.owner, id)?.document(), f.document);
                    members += 1;
                    Ok(())
                },
                || draft.source_step(&cancel()),
            )
            .unwrap();
        let discovery = work.spent.load(Ordering::Relaxed) - probe_before;
        drop(draft);
        costs.push((discovery, members));
        let outcome = f.prepare(&cancel()).unwrap().publish().finish();
        assert_eq!(outcome.document(), f.document);
        assert!(outcome.withdrawal_cssom_snapshot().sheet(&f.sheet).is_ok());
    }
    assert_eq!(
        costs[0], costs[1],
        "actual discovery work and current participants follow only the selected Document's maintained group"
    );
}

#[test]
fn remaining_same_document_root_refuses_but_other_document_root_and_shadow_selection_are_exact() {
    let mut same = Fixture::with_roots(true, 0, false, Some(false));
    let selected = same.root.root();
    let remaining = same.other_root.as_ref().unwrap().root();
    assert!(same.root.snapshot().unwrap().sheets().is_empty());
    assert!(
        same.other_root
            .as_ref()
            .unwrap()
            .snapshot()
            .unwrap()
            .sheets()
            .is_empty()
    );
    assert_eq!(
        same.prepare(&cancel()).unwrap_err(),
        DomHostedError::PendingDocumentObligations
    );
    assert!(same.dom.style_cssom_roots.get(selected).is_some());
    assert!(same.dom.style_cssom_roots.get(remaining).is_some());
    assert!(same.cleanup.is_some());

    let mut other = Fixture::with_roots(true, 0, true, Some(true));
    let selected = other.root.root();
    assert_ne!(selected, other.document.node());
    let independent = other.other_root.as_ref().unwrap();
    let other_node = independent.root();
    assert_eq!(
        other
            .lease
            .prepare_style_root_withdrawal(
                &mut other.dom,
                &other.source,
                other.cleanup.as_ref().unwrap(),
                independent,
                &cancel()
            )
            .unwrap_err(),
        DomHostedError::Dom(DomError::StyleReceiptMismatch)
    );
    let outcome = other.prepare(&cancel()).unwrap().publish().finish();
    assert_eq!(outcome.root(), selected);
    assert_eq!(outcome.document(), other.document);
    assert!(other.dom.style_cssom_roots.get(selected).is_none());
    assert!(other.dom.style_cssom_roots.get(other_node).is_some());
    other
        .lease
        .accept_publication(
            &mut other.dom,
            &mut other.source,
            &mut other.cleanup,
            &cancel(),
        )
        .unwrap();
    other
        .lease
        .finish(&mut other.dom, &mut other.source, &cancel())
        .unwrap();
    assert!(other.dom.style_cssom_roots.get(other_node).is_some());
}

#[test]
fn actual_issuer_revision_change_refuses_stale_cleanup_without_withdrawing_custody() {
    let mut f = Fixture::new(true, 0);
    let stamp = f.dom.snapshot().stamp();
    let old_revision = f.source.revision().unwrap();
    let work = f.work();
    let spent = work.spent.load(Ordering::Relaxed);
    // This real issuer action changes its revision independently of the DOM;
    // neither an old publication nor a same-number root selector repairs it.
    let _additional_host = f.source.issue_host(&cancel()).unwrap();
    assert_ne!(f.source.revision().unwrap(), old_revision);
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::StaleBasis
    );
    assert_eq!(f.dom.snapshot().stamp(), stamp);
    assert!(f.dom.style_cssom_roots.get(f.root.root()).is_some());
    assert!(f.cleanup.is_some());
    assert!(Arc::ptr_eq(&work, &f.work()));
    assert!(work.spent.load(Ordering::Relaxed) >= spent);
}

#[test]
fn actual_parked_source_keeps_its_mandatory_tail_and_original_cleanup_custody() {
    let mut f = Fixture::new(true, 0);
    let mut setup = DomBatch::new();
    let style = setup.create_element_ns(f.document, crate::Namespace::html(), "style");
    let text = setup.create_text(f.document, "b {}");
    setup.insert_before(&text, &style, None);
    let style = f
        .dom
        .apply(setup, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let parent = f
        .dom
        .snapshot()
        .document(f.document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut source = f
        .dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, parent, None),
            source_limits(),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        source.advance(&cancel()).unwrap(),
        DomSourceProgress::Request(DomSourceRequest::Style(_))
    ));
    // The genuine exposed request parks on Drop. No source row, task, or
    // processing flag is manufactured to simulate its mandatory continuation.
    drop(source);
    let pending = f.dom.pending_source_edit().unwrap();
    assert!(pending.has_published());
    let work = f.work();
    let spent = work.spent.load(Ordering::Relaxed);
    assert_eq!(
        f.prepare(&cancel()).unwrap_err(),
        DomHostedError::Dom(DomError::SourceOperationPending)
    );
    assert_eq!(f.dom.pending_source_edit().unwrap(), pending);
    assert_eq!(work.spent.load(Ordering::Relaxed), spent);
    assert!(f.dom.style_cssom_roots.get(f.root.root()).is_some());
    assert!(f.cleanup.is_some());
}
