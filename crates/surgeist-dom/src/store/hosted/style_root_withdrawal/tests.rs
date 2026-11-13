//! Real direct-DOM composition. Trusted publications model the host's existing
//! native protocol; no Runtime Window/task bank or event receipt is fabricated.
use super::*;
use crate::DomSourceEventClockMode::ContextFree;
use crate::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest};
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_css::{CssParser, CssParserContext};
use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 50_000_000, 64, 4096, 4096, 1)
}
fn constructor() -> (Dom, DomHostedSource, DocumentId) {
    constructor_with_meta_work(0)
}
fn constructor_with_meta_work(meta_work: u64) -> (Dom, DomHostedSource, DocumentId) {
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
        .unwrap()
        .with_meta_processing_work(meta_work),
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

#[test]
fn hosted_cleanup_waits_for_selectedcontent_when_its_select_was_adopted_away() {
    let cancel = cancel();
    let (mut dom, mut source, hosted) = constructor();
    dom.set_limits(
        dom.limits()
            .with_select_processing(8, 1_000_000, 1_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let closed = setup.create_document();
    let html = surgeist_html::values::Namespace::html;
    let root = setup.create_element_ns(hosted, html(), "div");
    let select = setup.create_element_ns(hosted, html(), "select");
    let target = setup.create_element_ns(hosted, html(), "selectedcontent");
    setup.insert_before(&select, &root, None);
    setup.insert_before(&target, &root, None);
    let commit = dom.apply(setup, &cancel).unwrap();
    let closed = commit.document(&closed).unwrap();
    let select = commit.created(&select).unwrap().unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let mut move_edit = dom
        .begin_source_edit(
            DomSourceEdit::move_before(target, select, None),
            source_limits(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        move_edit.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(move_edit);

    let mut adopt = DomBatch::new();
    adopt.detach(target);
    adopt.adopt(select, closed.node());
    dom.apply(adopt, &cancel).unwrap();
    assert_eq!(
        dom.snapshot().node(target).unwrap().owner_document(),
        Some(hosted)
    );
    assert_eq!(
        dom.snapshot().node(select).unwrap().owner_document(),
        Some(closed)
    );

    let navigable = dom
        .snapshot()
        .document(hosted)
        .unwrap()
        .navigable()
        .unwrap();
    let mut invocation = dom
        .begin_hosted_top_level_destruction(&mut source, navigable, &cancel)
        .unwrap();
    let mut lease = invocation.cleanup(&mut dom, &mut source, &cancel).unwrap();
    let DomHostedDestructionStep::EnqueueParentTask(request) = lease.step(&dom).unwrap() else {
        panic!("actual destruction task");
    };
    let mut queued = Some(source.seal_destruction_queued(request, &cancel).unwrap());
    lease
        .accept_publication(&mut dom, &mut source, &mut queued, &cancel)
        .unwrap();
    let DomHostedDestructionStep::Cleanup(request) = lease.step(&dom).unwrap() else {
        panic!("actual cleanup request");
    };
    let mut cleanup = Some(source.seal_top_level_cleanup(request, &cancel).unwrap());
    let before = dom.snapshot().stamp();
    assert_eq!(
        lease.accept_publication(&mut dom, &mut source, &mut cleanup, &cancel),
        Err(DomHostedError::PendingDocumentObligations)
    );
    assert!(cleanup.is_some());
    assert_eq!(dom.snapshot().stamp(), before);

    let checkpoint = crate::DomNativeCheckpointLimits::new(
        1_000_000,
        1_000_000,
        crate::DomSlotQueryLimits::new(32, 32).unwrap(),
    )
    .unwrap();
    let crate::DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("actual pair task");
    };
    assert_eq!(
        (task.select(), task.selectedcontent()),
        (select, Some(target))
    );
    dom.complete_select_task(&task, checkpoint, &cancel)
        .unwrap();
    lease
        .accept_publication(&mut dom, &mut source, &mut cleanup, &cancel)
        .unwrap();
    assert!(cleanup.is_none());
    assert!(matches!(
        lease.step(&dom).unwrap(),
        DomHostedDestructionStep::Complete
    ));
}

fn publish_style_sheet(
    lease: &mut crate::DomSourceLease<'_>,
    root: &DomStyleCssomRoot,
    create: crate::DomStyleRequest,
) -> (
    surgeist_cssom::CssomSheetId,
    crate::DomStyleNotificationToken,
) {
    lease.bind_cssom_root(&create, root, &cancel()).unwrap();
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
    let body = CssParser::new(parser, Default::default())
        .parse_sheet_utf16(
            create.source().text().clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = CssMediaQueryList::parse_utf16(
        &CssParser::new(parser, Default::default()),
        Arc::new(Vec::new()),
        CssUtf16InputLimits::default(),
        || Ok::<_, ()>(()),
    )
    .unwrap();
    let mut inputs = CssomSheetInputs::external(version, true);
    inputs.owner = Some("direct-style".into());
    if create.source().root_kind() == crate::DomStyleRootKind::Document
        && let Some(title) = create.source().title()
    {
        inputs.title = Arc::new(title.code_units().to_vec());
    }
    let mut batch = root
        .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
        .unwrap();
    let construction = batch
        .create_parsed_sheet_with_media_reports(body, media, inputs)
        .unwrap();
    let commit = root.commit(&mut Some(batch)).unwrap();
    let sheet = commit.sheet(construction.ticket()).unwrap().clone();
    let notification = lease
        .accept_cssom_publication(
            &create,
            commit,
            root.snapshot().unwrap(),
            Some(construction.ticket()),
        )
        .unwrap()
        .unwrap();
    (sheet, notification)
}

#[test]
fn exact_document_load_stops_before_looking_up_the_expired_global() {
    use crate::event::{ERROR_EVENT, LOAD_EVENT};
    use crate::{DomEventParent, DomEventParentQuery};

    let (dom, source, document) = constructor();
    let node = document.node();
    let parent = |kind| {
        dom.dispatch_event_parent(
            &dom.snapshot(),
            DomEventParentQuery::new(node, node, node, kind, false),
            &cancel(),
            &mut || Ok::<(), DomHostedError>(()),
        )
    };
    let global = dom.relevant_global(node).unwrap().unwrap();
    assert_eq!(parent(ERROR_EVENT).unwrap(), DomEventParent::Global(global),);

    drop(source);

    assert_eq!(parent(ERROR_EVENT), Err(DomHostedError::SourceExpired));
    assert_eq!(parent(LOAD_EVENT), Ok(DomEventParent::None));
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
    let style =
        setup.create_element_ns(document, surgeist_html::values::Namespace::html(), "style");
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
    let (sheet, mut notification) = publish_style_sheet(&mut lease, &root, create);
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
    // This direct host has no dispatched Rust callbacks. Its native
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
        // A detached participant belongs to the selected Document group.
        batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "style");
        for _ in 0..unrelated {
            let other = batch.create_document();
            batch.create_element_ns(&other, surgeist_html::values::Namespace::html(), "style");
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
            let host =
                setup.create_element_ns(document, surgeist_html::values::Namespace::html(), "div");
            setup.insert_before(&host, document_element, None);
            let shadow = setup.attach_shadow(
                &host,
                surgeist_html::construction::HtmlShadowRootProperties::new(
                    surgeist_html::construction::HtmlShadowRootMode::Open,
                    surgeist_html::construction::HtmlSlotAssignment::Manual,
                    false,
                    true,
                    false,
                ),
            );
            dom.apply(setup, &cancel())
                .unwrap()
                .attached_shadow_root(&shadow)
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
    // Independent create/remove publications reach the same numeric
    // revision; that does not grant the canonical producer's owner identity.
    let parser = CssParserContext::default();
    let body = CssParser::new(parser, Default::default())
        .parse_sheet_utf16(
            Arc::new("b {}".encode_utf16().collect()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = CssMediaQueryList::parse_utf16(
        &CssParser::new(parser, Default::default()),
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
    let style = setup.create_element_ns(
        f.document,
        surgeist_html::values::Namespace::html(),
        "style",
    );
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
    // The exposed request parks on Drop. No source row, task, or
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

#[test]
fn trusted_attribute_sink_parks_for_its_registered_global() {
    use crate::{
        DomDetachedNodeCreation, DomNodeRetirementLimits, DomSourceAttributeSink,
        DomSourceAttributeVerificationOutcome,
    };
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html::security::csp::{HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage};
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let source_limits =
        || DomSourceLimits::new(64_000_000, 1_000_000, 50_000_000, 64, 4096, 4096, 4);
    dom.set_limits(dom.limits().with_dispatch_custody(
        crate::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    ))
    .unwrap();
    let snapshot = dom.snapshot();
    let navigable = snapshot.document(document).unwrap().navigable().unwrap();
    let global = snapshot.hosted_navigable(navigable).unwrap().global();

    let context = source
        .document_constructor_context(&dom, global, &cancel)
        .unwrap();
    let constructed = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::construct_document(context),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("the standard Document constructor completes");
        };
        let Some(crate::DomSourceOutput::Document(constructed)) = done.output() else {
            panic!("the constructor returns the new Document");
        };
        constructed
    };
    let mut setup = DomBatch::new();
    let inert = setup.create_document();
    let element = setup.create_element_ns(&inert, Namespace::html(), "div");
    setup.adopt(&element, constructed);
    let element = dom
        .apply(setup, &cancel)
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap();
    let facts = crate::DomDocumentScriptingFacts {
        engine: HtmlScriptingEngine::Supported,
        policy: HtmlScriptingPolicy {
            user_allows: true,
            automation_allows: true,
        },
    };
    let backing = source
        .retain_runtime_payload(
            &mut dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel,
        )
        .unwrap();
    let (_binding, registration) = source
        .attach_document_script_host(&mut dom, global, document, facts, backing, &cancel)
        .unwrap();
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1000);
    let inbox = dom.node_retirement_inbox(watch_limits, &cancel).unwrap();

    let before = dom.snapshot();
    let receiver = before.node(element).unwrap();
    assert_eq!(receiver.owner_document(), Some(constructed));
    assert_eq!(receiver.relevant_global().unwrap(), None);
    assert_eq!(
        before
            .node(constructed.node())
            .unwrap()
            .relevant_global()
            .unwrap(),
        Some(global)
    );
    let proposed = crate::DocumentString::from_utf16(vec![0xd800, 0, 0x61]);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(element, "onclick", proposed.clone()),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the constructed Document supplies the registered verification Global");
    };
    assert_eq!(request.element(), element);
    assert_eq!(request.global(), global);
    assert_eq!(
        request.sink(),
        DomSourceAttributeSink::EventHandler("onclick")
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert_eq!(
        lease
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        None
    );
    assert!(!lease.status().has_published());
    assert_eq!(
        lease.status().phase(),
        crate::DomSourcePhase::AttributeVerification
    );
    drop(lease);

    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel)
        .unwrap();
    assert!(
        matches!(lease.advance(&cancel).unwrap(), DomSourceProgress::Request(DomSourceRequest::AttributeVerification(retry)) if retry == request)
    );
    assert_eq!(
        lease
            .attribute_verification_input(request, &registration, &cancel)
            .unwrap(),
        &proposed
    );
    let policy = lease
        .attribute_verification_policy(request, &registration, &cancel)
        .unwrap();
    assert_eq!(policy.document(), document);
    assert_ne!(policy.document(), constructed);
    assert!(matches!(
        policy.state(),
        crate::DomDocumentCspState::ConstructorInitialEmpty
    ));
    let spent = lease.status().spent_work();
    policy.step(&cancel).unwrap();
    assert_eq!(lease.status().spent_work(), spent + 1);
    let read = HtmlCspLayout::processing(
        3,
        HtmlCspStorage::default(),
        HtmlCspDisposalWork::default(),
        0,
    )
    .unwrap();
    policy.admit_csp_read(read, &cancel).unwrap();
    assert_eq!(lease.status().spent_work(), spent + 4);
    let retain = HtmlCspLayout::processing(
        2,
        HtmlCspStorage::try_new(2, 1, 1).unwrap(),
        HtmlCspDisposalWork::default(),
        0,
    )
    .unwrap();
    assert_eq!(
        policy.admit_csp_read(retain, &cancel),
        Err(DomError::HtmlChargeProtocol)
    );
    assert_eq!(lease.status().spent_work(), spent + 4);
    let scope = lease
        .open_attribute_verification_callback(request, &registration, &cancel)
        .unwrap();
    assert!(matches!(
        lease.open_attribute_verification_callback(request, &registration, &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    let mut early = Some(DomSourceAttributeVerificationOutcome::Rejected);
    assert_eq!(
        lease.accept_attribute_verification(request, &registration, &mut early, &cancel),
        Err(DomError::InvalidSourceOperation)
    );
    assert!(early.is_some());
    assert!(matches!(
        lease.retain_attribute_verification_payload(request, &registration, 8, 4, &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    drop(lease);
    dom.assert_attribute_callback_unit_peak(request, &registration, &cancel);
    let spent = dom.pending_source_edit().unwrap().spent_work();
    assert_eq!(
        dom.source_attribute_callback_step(request, &registration, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
    assert_eq!(dom.pending_source_edit().unwrap().spent_work(), spent);
    dom.source_attribute_callback_step(request, &registration, &cancel)
        .unwrap();
    assert!(dom.pending_source_edit().unwrap().spent_work() > spent);
    assert_eq!(
        dom.dispatch_relevant_global(constructed.node(), &cancel)
            .unwrap(),
        Some(global)
    );
    let snapshot = dom.snapshot();
    let parent = dom.pending_source_edit().unwrap();
    let before_work = parent.spent_work();
    assert!(matches!(
        dom.create_detached_with_node_retirement_watch(
            constructed,
            DomDetachedNodeCreation::Attribute("callback-attr".into()),
            (&inbox, 8),
            (source_limits(), DomNodeRetirementLimits::new(16_000_000, 0)),
            &cancel,
        ),
        Err(DomError::NodeRetirementWorkLimit { limit: 0 })
    ));
    let current = dom.pending_source_edit().unwrap();
    assert_eq!(current.operation(), parent.operation());
    assert_eq!(current.phase(), parent.phase());
    assert!(current.spent_work() > before_work);
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
    let (_, created, _watch) = dom
        .create_detached_with_node_retirement_watch(
            constructed,
            DomDetachedNodeCreation::Attribute("callback-attr".into()),
            (&inbox, 8),
            (source_limits(), watch_limits),
            &cancel,
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    assert_eq!(
        dom.snapshot().attr(created).unwrap().owner_document(),
        constructed
    );
    let mut nested = dom
        .begin_source_edit(
            DomSourceEdit::set_native_attribute(element, "onclick", "created in callback"),
            source_limits(),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        nested.status().parent_operation(),
        Some(request.operation())
    );
    assert!(matches!(
        nested.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(nested);
    let inserted = dom
        .snapshot()
        .node(element)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id();
    drop(scope);
    assert_eq!(
        dom.source_attribute_callback_step(request, &registration, &cancel),
        Err(DomError::InvalidSourceOperation)
    );
    assert_eq!(dom.source_dispatch_blocker(), Some(request.operation()));
    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel)
        .unwrap();
    assert!(matches!(
        lease.open_attribute_verification_callback(request, &registration, &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    assert!(matches!(
        lease.retain_attribute_verification_payload(request, &registration, 1, 1, &cancel),
        Err(DomError::RepresentationExhausted)
    ));
    let report = lease
        .retain_attribute_verification_payload(request, &registration, 8, 4, &cancel)
        .unwrap();
    drop(report);
    let verified = crate::DocumentString::from_utf16(vec![0xdc00, 0x62]);
    assert_eq!(
        lease
            .attribute_verification_policy(request, &registration, &cancel)
            .unwrap()
            .document(),
        document
    );
    let mut reply = Some(DomSourceAttributeVerificationOutcome::Value(
        verified.clone(),
    ));
    let interrupted = AtomicBool::new(true);
    assert_eq!(
        lease.accept_attribute_verification(request, &registration, &mut reply, &interrupted),
        Err(DomError::Cancelled)
    );
    assert!(reply.is_some());
    lease
        .accept_attribute_verification(request, &registration, &mut reply, &cancel)
        .unwrap();
    assert!(reply.is_none());
    let mut duplicate = Some(DomSourceAttributeVerificationOutcome::Rejected);
    assert_eq!(
        lease.accept_attribute_verification(request, &registration, &mut duplicate, &cancel),
        Err(DomError::InvalidSourceOperation)
    );
    assert!(duplicate.is_some());
    lease.request_cancel();
    assert!(!lease.status().cancellation_requested());
    let meta_limits =
        crate::DomMetaAtomicLimits::new(64_000_000, 1_000_000, 4_000_000, 256).unwrap();
    let mut hold = lease.begin_meta_runtime(meta_limits, &cancel).unwrap();
    hold.request_cancel().unwrap();
    drop(hold);
    assert!(!lease.status().cancellation_requested());
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the accepted verification must publish its original result");
    };
    assert!(!done.cancellation_requested());
    assert_eq!(
        lease
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&verified)
    );
    assert_eq!(
        lease
            .snapshot()
            .node(element)
            .unwrap()
            .get_attribute_node(&"onclick".into())
            .unwrap()
            .id(),
        inserted
    );
    drop(lease);

    for (outcome, error) in [
        (
            DomSourceAttributeVerificationOutcome::Rejected,
            DomError::SourceAttributeVerificationRejected,
        ),
        (
            DomSourceAttributeVerificationOutcome::Abrupt,
            DomError::SourceAttributeVerificationAbrupt,
        ),
    ] {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_attribute(element, "onclick", "new"),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(current)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the new operation reaches its own verification request");
        };
        let mut outcome = Some(outcome);
        assert_eq!(
            lease.accept_attribute_verification(request, &registration, &mut outcome, &cancel),
            Err(DomError::InvalidSourceOperation)
        );
        lease
            .accept_attribute_verification(current, &registration, &mut outcome, &cancel)
            .unwrap();
        let DomSourceProgress::Failed(failure) = lease.advance(&cancel).unwrap() else {
            panic!("a verification failure is terminal");
        };
        assert_eq!(failure.error(), &error);
        assert_eq!(
            lease
                .snapshot()
                .node(element)
                .unwrap()
                .attribute(&"onclick".into()),
            Some(&verified)
        );
    }

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute_value(inserted, "before detach"),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("an initially attached Attr reaches verification");
    };
    let scope = lease
        .open_attribute_verification_callback(request, &registration, &cancel)
        .unwrap();
    drop(lease);
    let mut nested = dom
        .begin_source_edit(
            DomSourceEdit::remove_attribute_node(element, inserted),
            source_limits(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        nested.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(nested);
    drop(scope);
    let detached_value = crate::DocumentString::from_utf16(vec![0x63, 0xd800]);
    let mut reply = Some(DomSourceAttributeVerificationOutcome::Value(
        detached_value.clone(),
    ));
    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel)
        .unwrap();
    lease
        .accept_attribute_verification(request, &registration, &mut reply, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    let snapshot = lease.snapshot();
    let detached = snapshot.attr(inserted.node()).unwrap();
    assert_eq!(detached.owner_element(), None);
    assert_eq!(detached.value(), &detached_value);
    assert_eq!(
        snapshot.node(element).unwrap().attribute(&"onclick".into()),
        None
    );
    drop(lease);
    dom.assert_interrupted_attribute_raises_for_child(
        element,
        &registration,
        source_limits(),
        &cancel,
    );
}

#[test]
fn registered_attribute_verification_precedes_attr_writes_and_attachment_guards() {
    use crate::{DomAttributeAttachment, DomSourceAttributeVerificationOutcome, DomSourceOutput};
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let snapshot = dom.snapshot();
    let navigable = snapshot.document(document).unwrap().navigable().unwrap();
    let global = snapshot.hosted_navigable(navigable).unwrap().global();
    let backing = source
        .retain_runtime_payload(
            &mut dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel,
        )
        .unwrap();
    let (_, registration) = source
        .attach_document_script_host(
            &mut dom,
            global,
            document,
            crate::DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel,
        )
        .unwrap();

    let mut setup = DomBatch::new();
    let first = setup.create_element_ns(document, Namespace::html(), "div");
    let second = setup.create_element_ns(document, Namespace::html(), "div");
    let incoming = setup.create_attribute(document, "onclick");
    setup.set_attribute(&first, "onclick", "original");
    let commit = dom.apply(setup, &cancel).unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let incoming = dom
        .snapshot()
        .attr(commit.created(&incoming).unwrap().unwrap())
        .unwrap()
        .id();
    let handler = dom
        .snapshot()
        .node(first)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id();
    let verified = crate::DocumentString::from_utf16(vec![0xd800, 0, 0x62]);

    for (edit, receiver, input, output) in [
        (
            DomSourceEdit::set_attribute_value(handler, "argument"),
            first,
            "argument".into(),
            None,
        ),
        (
            DomSourceEdit::set_text_content(handler.node(), "text"),
            first,
            "text".into(),
            None,
        ),
        (
            DomSourceEdit::attach_attribute(first, handler),
            first,
            verified.clone(),
            Some(DomAttributeAttachment::Unchanged(handler)),
        ),
        (
            DomSourceEdit::attach_attribute(second, handler),
            second,
            verified.clone(),
            None,
        ),
        (
            DomSourceEdit::attach_attribute(first, incoming),
            first,
            "".into(),
            Some(DomAttributeAttachment::Replaced(handler)),
        ),
    ] {
        let stamp = dom.snapshot().stamp();
        let mut lease = dom
            .begin_source_edit(edit, source_limits(), &cancel)
            .unwrap();
        let progress = lease.advance(&cancel);
        let Ok(DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request))) =
            progress
        else {
            panic!(
                "standard Attr operations must request verification before writing or checking ownership: {progress:?}"
            );
        };
        assert_eq!(request.element(), receiver);
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(
            lease
                .attribute_verification_input(request, &registration, &cancel)
                .unwrap(),
            &input
        );
        let reply = if output == Some(DomAttributeAttachment::Unchanged(handler)) {
            "must not replace".into()
        } else {
            verified.clone()
        };
        let mut reply = Some(DomSourceAttributeVerificationOutcome::Value(reply));
        lease
            .accept_attribute_verification(request, &registration, &mut reply, &cancel)
            .unwrap();
        let progress = lease.advance(&cancel).unwrap();
        if receiver == second {
            let DomSourceProgress::Failed(failure) = progress else {
                panic!("the original InUse error follows verification: {progress:?}");
            };
            assert_eq!(failure.error(), &DomError::InUseAttribute);
        } else {
            let DomSourceProgress::Complete(completion) = progress else {
                panic!("the verified mutation completes: {progress:?}");
            };
            assert_eq!(
                completion.output(),
                output.map(DomSourceOutput::AttributeAttachment)
            );
        }
        let snapshot = lease.snapshot();
        let current = snapshot
            .node(first)
            .unwrap()
            .get_attribute_node(&"onclick".into())
            .unwrap();
        assert_eq!(current.value(), &verified);
        let id = if output == Some(DomAttributeAttachment::Replaced(handler)) {
            incoming
        } else {
            handler
        };
        assert_eq!(current.id(), id);
    }
    let detached = dom.snapshot();
    let replaced = detached.attr(handler.node()).unwrap();
    assert_eq!(replaced.owner_element(), None);
    assert_eq!(replaced.value(), &verified);
}

#[test]
fn script_settings_from_an_expired_hosted_issuer_do_not_advance_source() {
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let snapshot = dom.snapshot();
    let document_view = snapshot.document(document).unwrap();
    let environment = document_view.metadata().document_environment();
    let association = environment.hosted.as_ref().unwrap();
    let global = association.global;
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];

    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let text = batch.create_text(document, "globalThis.ran = true");
    batch.insert_before(&text, &script, None);
    let script = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&script)
        .unwrap()
        .unwrap();
    let facts = crate::DomDocumentScriptingFacts {
        engine: HtmlScriptingEngine::Supported,
        policy: HtmlScriptingPolicy {
            user_allows: true,
            automation_allows: true,
        },
    };
    let backing = source
        .retain_runtime_payload(
            &mut dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel,
        )
        .unwrap();
    let (binding, _registration) = source
        .attach_document_script_host(&mut dom, global, document, facts, backing, &cancel)
        .unwrap();

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(script, body, None),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Script(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the original connected Script must request its live settings");
    };
    let observation = lease
        .observe_script_settings(request, &source, &binding, facts, &cancel)
        .unwrap();
    assert!(observation.enabled());
    drop(source);
    assert_eq!(lease.snapshot().stamp(), request.basis());
    assert_eq!(
        lease.accept_script_settings(observation, &cancel),
        Err(DomError::HostedSourceUnavailable)
    );
    let DomSourceProgress::Request(DomSourceRequest::Script(retry)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the original Script must still be waiting for its live settings");
    };
    assert_eq!(retry, request);
    assert_eq!(lease.snapshot().stamp(), request.basis());
}

#[test]
fn style_denial_before_a_disabled_script_completes_the_original_insertion() {
    use crate::{DomDocumentScriptingFacts, DomSourceScriptStep, DomStyleInlineCspObservation};
    use surgeist_html::elements::script::{HtmlScriptKind, HtmlScriptResult};
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let initial = dom.snapshot();
    let navigable = initial.document(document).unwrap().navigable().unwrap();
    let hosted = initial.hosted_navigable(navigable).unwrap();
    let global = hosted.global();
    let html = initial.node(document.node()).unwrap().children()[0];
    let body = initial.node(html).unwrap().children()[1];

    let mut batch = DomBatch::new();
    let wrapper = batch.create_element_ns(document, Namespace::html(), "div");
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let css = batch.create_text(document, "div { color: red }");
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let code = batch.create_text(document, "globalThis.ran = true");
    batch.insert_before(&css, &style, None);
    batch.insert_before(&style, &wrapper, None);
    batch.insert_before(&code, &script, None);
    batch.insert_before(&script, &wrapper, None);
    let setup = dom.apply(batch, &cancel).unwrap();
    let [wrapper, style, script] =
        [wrapper, style, script].map(|node| setup.created(&node).unwrap().unwrap());

    // This fixture drives the trusted canonical DOM protocol. Its current
    // scripting preference can differ from the facts at attachment time.
    let allowed = DomDocumentScriptingFacts {
        engine: HtmlScriptingEngine::Supported,
        policy: HtmlScriptingPolicy {
            user_allows: true,
            automation_allows: true,
        },
    };
    let backing = source
        .retain_runtime_payload(
            &mut dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel,
        )
        .unwrap();
    let (binding, _registration) = source
        .attach_document_script_host(&mut dom, global, document, allowed, backing, &cancel)
        .unwrap();
    let disabled = DomDocumentScriptingFacts {
        policy: HtmlScriptingPolicy {
            user_allows: false,
            ..allowed.policy
        },
        ..allowed
    };
    let (operation, prefix) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(wrapper, body, None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the first connected sibling must request Style CSP");
        };
        let operation = lease.operation();
        assert_eq!(policy.source().element(), style);
        assert_eq!(policy.source().operation(), operation);
        let denial = DomStyleInlineCspObservation::evaluated(
            policy.source(),
            Arc::new(vec!["style-src 'none'".into()]),
            1,
            false,
        );
        lease.accept_inline_csp(&policy, &denial, &cancel).unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Script(read)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the next connected sibling must request Script settings");
        };
        assert_eq!(read.operation(), operation);
        assert_eq!(read.step(), DomSourceScriptStep::ReadScripting);
        assert_eq!(read.subject(), Some(script));
        assert_eq!(read.relevant_global(), Some(global));
        let prefix = lease.snapshot();
        assert_eq!(read.basis(), prefix.stamp());
        assert_eq!(prefix.incomplete_source_operation(), Some(operation));
        assert_eq!(prefix.node(wrapper).unwrap().parent(), Some(body));
        assert!(
            !prefix
                .node(script)
                .unwrap()
                .as_html_script()
                .unwrap()
                .flags()
                .already_started()
        );

        let observed = lease
            .observe_script_settings(read, &source, &binding, disabled, &cancel)
            .unwrap();
        assert_eq!(observed.request(), read);
        assert!(!observed.enabled());
        lease.accept_script_settings(observed, &cancel).unwrap();
        assert_eq!(
            lease.source_script_step(read, &cancel),
            Err(DomError::InvalidSourceOperation)
        );
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        (operation, prefix)
    };
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    let mut lease = dom
        .resume_source_edit(operation, source_limits(), &cancel)
        .unwrap();
    let progress = lease
        .advance(&cancel)
        .expect("a consumed Style receipt must not strand the original Script tail");
    let DomSourceProgress::Complete(done) = progress else {
        panic!("the same insertion must complete after its disabled Script: {progress:?}");
    };
    assert_eq!(done.operation(), operation);
    drop(lease);

    assert!(dom.pending_source_edit().is_none());
    let final_snapshot = dom.snapshot();
    assert_eq!(final_snapshot.incomplete_source_operation(), None);
    assert_ne!(final_snapshot.stamp(), prefix.stamp());
    assert_eq!(final_snapshot.node(wrapper).unwrap().parent(), Some(body));
    assert_eq!(
        final_snapshot.node(wrapper).unwrap().children(),
        [style, script]
    );
    assert!(
        final_snapshot
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    let prepared = final_snapshot
        .node(script)
        .unwrap()
        .as_html_script()
        .unwrap();
    assert!(prepared.flags().already_started());
    assert_eq!(prepared.preparation_document(), Some(document));
    assert_eq!(prepared.script_type(), Some(HtmlScriptKind::Classic));
    assert_eq!(prepared.result(), HtmlScriptResult::Uninitialized);
}

#[test]
fn published_style_acknowledges_sheet_after_svg_refusal() {
    check_published_style_across_script_yield(StyleClose::Complete);
}

#[test]
fn published_style_retains_acknowledgment_through_svg_close_refusals() {
    check_published_style_across_script_yield(StyleClose::Retry);
}

#[test]
fn published_style_closes_after_an_independent_cssom_commit() {
    check_published_style_across_script_yield(StyleClose::RetryAfterCssomCommit);
}

#[test]
fn parked_style_refuses_a_changed_initial_disabled_effect() {
    check_published_style_across_script_yield(StyleClose::ChangedEffect);
}

#[test]
fn ordinary_script_close_refusal_keeps_the_original_style_receipt() {
    check_published_style_across_script_yield(StyleClose::OrdinaryRetry);
}

enum StyleClose {
    Complete,
    Retry,
    RetryAfterCssomCommit,
    ChangedEffect,
    OrdinaryRetry,
}

fn check_published_style_across_script_yield(scenario: StyleClose) {
    use crate::{DomDocumentScriptingFacts, DomSourceScriptStep, DomStyleInlineCspObservation};
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let initial = dom.snapshot();
    let navigable = initial.document(document).unwrap().navigable().unwrap();
    let global = initial.hosted_navigable(navigable).unwrap().global();
    let html = initial.node(document.node()).unwrap().children()[0];
    let parent = initial.node(html).unwrap().children()[1];
    let root = DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let empty = root.snapshot().unwrap();
    assert!(empty.sheets().is_empty());

    let mut batch = DomBatch::new();
    let wrapper = batch.create_element_ns(document, Namespace::html(), "div");
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    if matches!(scenario, StyleClose::ChangedEffect) {
        batch.set_attribute(&style, "title", "preferred");
    }
    let css = batch.create_text(document, "div { color: red }");
    batch.insert_before(&css, &style, None);
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let code = batch.create_text(document, "globalThis.ran = true");
    batch.insert_before(&code, &script, None);
    batch.insert_before(&style, &wrapper, None);
    batch.insert_before(&script, &wrapper, None);
    let svg = if matches!(scenario, StyleClose::OrdinaryRetry) {
        None
    } else {
        let detached = batch.create_html_document(document, None);
        let svg = batch.create_element_ns(&detached, Namespace::svg(), "script");
        batch.insert_before(&svg, &wrapper, None);
        Some(svg)
    };
    let setup = dom.apply(batch, &cancel).unwrap();
    let [wrapper, style, script] =
        [wrapper, style, script].map(|node| setup.created(&node).unwrap().unwrap());
    let svg = svg.map(|node| setup.created(&node).unwrap().unwrap());
    let facts = DomDocumentScriptingFacts {
        engine: HtmlScriptingEngine::Supported,
        policy: HtmlScriptingPolicy {
            user_allows: true,
            automation_allows: true,
        },
    };
    let backing = source
        .retain_runtime_payload(
            &mut dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel,
        )
        .unwrap();
    let (binding, registration) = source
        .attach_document_script_host(&mut dom, global, document, facts, backing, &cancel)
        .unwrap();
    let before = dom.snapshot();
    assert_eq!(before.node(wrapper).unwrap().parent(), None);
    let old_style = before.node(style).unwrap().as_style().unwrap().state();

    let (read, prefix, sheet, applied, mut notification) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(wrapper, parent, None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the first connected child must request Style policy");
        };
        assert_eq!(policy.source().element(), style);
        let operation = lease.operation();
        assert_eq!(policy.source().operation(), operation);
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the actual allowed Style must request its CSSOM sheet");
        };
        let (sheet, notification) = publish_style_sheet(&mut lease, &root, create);
        assert_eq!(
            notification.origin(),
            &crate::DomStyleNotificationOrigin::NativeSource(operation)
        );
        let applied = root.snapshot().unwrap();
        assert_eq!(applied.sheets(), std::slice::from_ref(&sheet));
        assert_ne!(applied.revision(), empty.revision());
        let DomSourceProgress::Request(DomSourceRequest::Script(read)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the next connected child must request Script settings");
        };
        assert_eq!(read.operation(), operation);
        assert_eq!(read.step(), DomSourceScriptStep::ReadScripting);
        assert_eq!(read.subject(), Some(script));
        assert_eq!(read.node_document(), Some(document));
        assert_eq!(read.relevant_global(), Some(global));
        let prefix = lease.snapshot();
        assert_eq!(prefix.stamp(), read.basis());
        assert_eq!(prefix.incomplete_source_operation(), Some(operation));
        assert_eq!(
            prefix.node(style).unwrap().as_style().unwrap().state(),
            old_style
        );
        (read, prefix, sheet, applied, notification)
    };
    let check_prefix = |snapshot: &crate::DomSnapshot| {
        assert_eq!(snapshot.node(wrapper).unwrap().parent(), Some(parent));
        let children = [Some(style), Some(script), svg]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(snapshot.node(wrapper).unwrap().children(), children);
        let script = snapshot.node(script).unwrap().as_html_script().unwrap();
        assert!(!script.flags().already_started());
        assert_eq!(script.script_type(), None);
        assert_eq!(script.preparation_document(), None);
        if let Some(svg) = svg {
            let svg = snapshot.node(svg).unwrap().as_svg_script().unwrap();
            assert_eq!(svg.origin(), crate::DomSvgScriptOrigin::Native);
            assert!(!svg.flags().already_processed());
        }
    };
    check_prefix(&prefix);
    let producer = if matches!(scenario, StyleClose::RetryAfterCssomCommit) {
        let mut batch = root.batch(CssomEditGuard::from_snapshot(&applied)).unwrap();
        let unrelated = batch
            .construct_sheet(
                surgeist_cssom::CssomConstructorDocument {
                    version: CssomInputVersion {
                        role: CssomInputRole::Document,
                        identity: "other".into(),
                        revision: 1,
                    },
                    identity: "other".into(),
                    base_location: "about:blank".into(),
                },
                Default::default(),
                &cancel,
            )
            .unwrap();
        let commit = root.commit(&mut Some(batch)).unwrap();
        let producer = root.snapshot().unwrap();
        assert_ne!(producer.revision(), applied.revision());
        assert!(
            producer
                .sheet(commit.sheet(unrelated.ticket()).unwrap())
                .is_ok()
        );
        assert_eq!(producer.sheets(), applied.sheets());
        assert_eq!(dom.snapshot().stamp(), prefix.stamp());
        producer
    } else if matches!(scenario, StyleClose::ChangedEffect) {
        let mut batch = root.batch(CssomEditGuard::from_snapshot(&applied)).unwrap();
        assert!(batch.set_disabled(&sheet, true).unwrap());
        root.commit(&mut Some(batch)).unwrap();
        let changed = root.snapshot().unwrap();
        assert_ne!(changed.revision(), applied.revision());
        assert_eq!(changed.sheets(), applied.sheets());
        assert!(changed.sheet(&sheet).unwrap().inputs().disabled);
        {
            let mut lease = dom
                .resume_source_edit(read.operation(), source_limits(), &cancel)
                .unwrap();
            assert!(matches!(
                lease.advance(&cancel),
                Err(DomError::StyleReceiptMismatch)
            ));
            let limits =
                crate::DomMetaAtomicLimits::new(64_000_000, 1_000_000, 4_000_000, 256).unwrap();
            let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
            assert!(matches!(
                hold.prepare(&[&root], None, ContextFree, &cancel),
                Err(crate::DomMetaAtomicError::Dom(
                    DomError::StyleReceiptMismatch
                ))
            ));
        }
        assert_eq!(dom.snapshot().stamp(), prefix.stamp());
        let mut batch = root.batch(CssomEditGuard::from_snapshot(&changed)).unwrap();
        assert!(batch.set_disabled(&sheet, false).unwrap());
        root.commit(&mut Some(batch)).unwrap();
        let restored = root.snapshot().unwrap();
        let mut lease = dom
            .resume_source_edit(read.operation(), source_limits(), &cancel)
            .unwrap();
        let limits =
            crate::DomMetaAtomicLimits::new(64_000_000, 1_000_000, 4_000_000, 256).unwrap();
        let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
        let caps = [&root];
        let crate::DomMetaSourceStep::Progress(progress) =
            hold.prepare(&caps, None, ContextFree, &cancel).unwrap()
        else {
            panic!("the exact restored effect must leave the original Script read parked");
        };
        assert!(matches!(
            progress.into_parts(),
            (DomSourceProgress::Request(DomSourceRequest::Script(again)), None) if again == read
        ));
        restored
    } else {
        applied.clone()
    };
    drop(registration);
    dom.release_document_script_registration(&binding);
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    let with_outputs = |outputs| {
        let limits = source_limits();
        DomSourceLimits::new(
            limits.retained_bytes(),
            limits.retained_string_code_units(),
            limits.work(),
            limits.phase_publications(),
            outputs,
            limits.published_native_identities(),
            limits.nested_operations(),
        )
    };
    if matches!(scenario, StyleClose::OrdinaryRetry) {
        let spent = dom.changes_since(before.stamp()).unwrap().changes().len();
        let mut lease = dom
            .resume_source_edit(read.operation(), with_outputs(spent), &cancel)
            .unwrap();
        let refused = lease.advance(&cancel);
        assert!(
            matches!(refused, Err(DomError::SourceOutputLimit)),
            "{refused:?}"
        );
        drop(lease);
        let unchanged = dom.snapshot();
        assert_eq!(unchanged.stamp(), prefix.stamp());
        assert_eq!(
            unchanged.incomplete_source_operation(),
            Some(read.operation())
        );
        assert_eq!(
            unchanged.node(style).unwrap().as_style().unwrap().state(),
            old_style
        );
        check_prefix(&unchanged);

        let mut batch = root.batch(CssomEditGuard::from_snapshot(&applied)).unwrap();
        assert!(batch.set_disabled(&sheet, true).unwrap());
        root.commit(&mut Some(batch)).unwrap();
        let changed = root.snapshot().unwrap();
        assert_ne!(changed.revision(), applied.revision());
        assert_eq!(changed.sheets(), applied.sheets());
        let mut lease = dom
            .resume_source_edit(read.operation(), source_limits(), &cancel)
            .unwrap();
        let retry = lease.advance(&cancel);
        assert!(
            matches!(retry, Err(DomError::StyleReceiptMismatch)),
            "retry must still qualify the exact original Style receipt: {retry:?}"
        );
        drop(lease);
        assert_eq!(dom.snapshot().stamp(), prefix.stamp());

        let mut batch = root.batch(CssomEditGuard::from_snapshot(&changed)).unwrap();
        assert!(batch.set_disabled(&sheet, false).unwrap());
        root.commit(&mut Some(batch)).unwrap();
        let restored = root.snapshot().unwrap();
        let mut lease = dom
            .resume_source_edit(read.operation(), source_limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Complete(completion) = lease.advance(&cancel).unwrap() else {
            panic!("the original operation must complete after restoring the accepted effect");
        };
        assert_eq!(completion.operation(), read.operation());
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        let closed = dom.snapshot();
        assert_eq!(closed.incomplete_source_operation(), None);
        let association = closed
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap();
        assert_eq!(association.id(), &sheet);
        assert_eq!(association.revision(), applied.revision());
        assert_eq!(association.association(), notification.association());
        assert_eq!(root.snapshot().unwrap().revision(), restored.revision());
        let script = closed.node(script).unwrap().as_html_script().unwrap();
        assert!(script.flags().already_started());
        assert_eq!(script.preparation_document(), Some(document));
        return;
    }
    let limits = if !matches!(scenario, StyleClose::Complete) {
        let spent = dom.changes_since(before.stamp()).unwrap().changes().len();
        let status = dom.pending_source_edit().unwrap();
        let mut lease = dom
            .resume_source_edit(read.operation(), with_outputs(spent + 1), &cancel)
            .unwrap();
        let refused = lease.advance(&cancel);
        assert!(
            matches!(refused, Err(DomError::SourceOutputLimit)),
            "{refused:?}"
        );
        drop(lease);
        let cleanup = dom.pending_source_edit().unwrap();
        assert_eq!(cleanup.operation(), read.operation());
        assert_eq!(cleanup.phase(), crate::DomSourcePhase::PendingCleanup);
        assert_eq!(
            cleanup.cleanup_cause(),
            Some(crate::DomSourceCleanupCause::SvgScriptProcessingUnavailable)
        );
        assert_eq!(cleanup.published_phases(), status.published_phases());
        assert!(cleanup.spent_work() > status.spent_work());
        let unchanged = dom.snapshot();
        assert_eq!(unchanged.stamp(), prefix.stamp());
        assert_eq!(
            unchanged.incomplete_source_operation(),
            Some(read.operation())
        );
        assert_eq!(
            unchanged.node(style).unwrap().as_style().unwrap().state(),
            old_style
        );
        check_prefix(&unchanged);
        assert_eq!(root.snapshot().unwrap().revision(), producer.revision());
        let enough = with_outputs(spent + 2);
        let mut lease = dom
            .resume_source_edit(read.operation(), enough, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&AtomicBool::new(true)),
            Err(DomError::Cancelled)
        ));
        drop(lease);
        let cancelled = dom.pending_source_edit().unwrap();
        assert_eq!(cancelled.operation(), cleanup.operation());
        assert_eq!(cancelled.cleanup_cause(), cleanup.cleanup_cause());
        assert_eq!(cancelled.published_phases(), cleanup.published_phases());
        assert_eq!(dom.snapshot().stamp(), prefix.stamp());
        enough
    } else {
        source_limits()
    };
    let mut lease = dom
        .resume_source_edit(read.operation(), limits, &cancel)
        .unwrap();
    let refused = lease.advance(&cancel);
    assert!(
        matches!(refused, Err(DomError::SvgScriptProcessingUnavailable)),
        "the actual Disabled read must reach the later SVG refusal: {refused:?}"
    );
    drop(lease);

    let closed = dom.snapshot();
    check_prefix(&closed);
    let current = root.snapshot().unwrap();
    assert_eq!(current.revision(), producer.revision());
    assert_eq!(current.sheets(), applied.sheets());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "published Style must settle its original CSSOM acknowledgment after SVG refusal: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    let association = closed
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert_eq!(association.id(), &sheet);
    assert_eq!(association.revision(), applied.revision());
    assert_eq!(association.root(), document.node());
    assert_eq!(association.association(), notification.association());
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [
            crate::DomChange::ElementState(style),
            crate::DomChange::SourceOperation
        ],
    );
    assert!(matches!(
        dom.finish_style_notification(
            &mut notification,
            crate::DomStyleNotificationDelivery::Native,
            1,
            &cancel
        )
        .unwrap(),
        crate::DomStyleNotificationCompletion::Complete(_),
    ));
}

#[test]
fn fullscreen_handoff_requalifies_the_original_style_removal() {
    use surgeist_cssom::CssomSheetListTarget;
    use surgeist_html::elements::style::HtmlStyleUpdateAction;
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let limits = DomLimits::new(
        64,
        100_000,
        256,
        128,
        128,
        16,
        4_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_fullscreen_exits(4)
    .unwrap()
    .with_fullscreen_events(4)
    .unwrap();
    let (mut dom, source, document, fullscreen) =
        crate::store::source::operation::fullscreen::native_fixture(limits);
    let parent = dom.snapshot().node(fullscreen).unwrap().parent().unwrap();

    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    batch.set_attribute(&script, "type", "application/x-unrecognized");
    let text = batch.create_text(document, "globalThis.ran = true");
    batch.insert_before(&text, &script, None);
    batch.insert_before(&script, parent, None);
    let script = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&script)
        .unwrap()
        .unwrap();
    let (root, sheet, attached) =
        style_resource(&mut dom, document, script, document.node(), false);
    let style = dom.snapshot().node(script).unwrap().children()[1];
    let mut batch = DomBatch::new();
    batch.set_attribute(script, "type", "text/javascript");
    dom.apply(batch, &cancel).unwrap();
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    let before = dom.snapshot();
    let expected = before
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert_eq!(expected.id(), &sheet);
    assert_eq!(before.node(style).unwrap().parent(), Some(script));
    assert!(
        !before
            .node(script)
            .unwrap()
            .as_html_script()
            .unwrap()
            .flags()
            .already_started()
    );
    assert!(
        before
            .node(fullscreen)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );

    let (read, removed) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_child(style, parent, fullscreen),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(remove)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the original removal of Style from Script must reach CSSOM");
        };
        assert_eq!(
            remove.action(),
            crate::DomStyleAction::Update(HtmlStyleUpdateAction::RemoveAssociatedSheet)
        );
        assert_eq!(remove.source().element(), style);
        assert_eq!(remove.sheet(), Some(expected));
        let remove = lease
            .qualify_cssom(&remove, root.root(), &attached)
            .unwrap();
        let mut batch = root
            .batch(CssomEditGuard::from_snapshot(&attached))
            .unwrap();
        batch.remove_sheet(&sheet).unwrap();
        let commit = root.commit(&mut Some(batch)).unwrap();
        let removed = root.snapshot().unwrap();
        assert!(removed.sheets().is_empty());
        assert!(
            lease
                .accept_cssom_publication(&remove, commit, removed.clone(), None)
                .unwrap()
                .is_none()
        );
        let DomSourceProgress::Request(DomSourceRequest::Script(read)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the old parent must ask for the actual Script settings");
        };
        assert_eq!(read.subject(), Some(script));
        assert_eq!(read.operation(), lease.operation());
        (read, removed)
    };
    drop(registration);
    dom.release_document_script_registration(&binding);

    let mut lease = dom
        .resume_source_edit(read.operation(), source_limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(handoff)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the actual disabled Script must reach the original fullscreen handoff");
    };
    assert_eq!(handoff.operation(), read.operation());
    assert_eq!(handoff.document(), document);
    let prefix = lease.snapshot();
    assert!(
        prefix
            .node(fullscreen)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );

    let mut batch = root.batch(CssomEditGuard::from_snapshot(&removed)).unwrap();
    assert!(
        batch
            .set_sheet_list_membership(
                removed.sheet_list(),
                &[CssomSheetListTarget::Existing(&sheet)],
            )
            .unwrap()
    );
    root.commit(&mut Some(batch)).unwrap();
    let changed = root.snapshot().unwrap();
    assert_eq!(changed.sheets(), std::slice::from_ref(&sheet));
    assert_ne!(changed.revision(), removed.revision());

    match lease.prepare_fullscreen_exit_handoff(&handoff, &source, &cancel) {
        Err(DomError::StyleReceiptMismatch) => {}
        Err(other) => {
            panic!("the original Style effect must refuse at the fullscreen fence: {other:?}")
        }
        Ok(prepared) => {
            let job = prepared.publish().unwrap();
            panic!(
                "the fullscreen handoff published after invalidating the original Style receipt: {job:?}"
            );
        }
    }
    assert_eq!(lease.snapshot().stamp(), prefix.stamp());
    assert_eq!(
        lease.snapshot().incomplete_source_operation(),
        Some(read.operation())
    );
    let mut batch = root.batch(CssomEditGuard::from_snapshot(&changed)).unwrap();
    assert!(
        batch
            .set_sheet_list_membership(changed.sheet_list(), &[])
            .unwrap()
    );
    root.commit(&mut Some(batch)).unwrap();
    let restored = root.snapshot().unwrap();

    let prepared = lease
        .prepare_fullscreen_exit_handoff(&handoff, &source, &cancel)
        .unwrap();
    let mut batch = root
        .batch(CssomEditGuard::from_snapshot(&restored))
        .unwrap();
    assert!(
        batch
            .set_sheet_list_membership(
                restored.sheet_list(),
                &[CssomSheetListTarget::Existing(&sheet)],
            )
            .unwrap()
    );
    root.commit(&mut Some(batch)).unwrap();
    let changed = root.snapshot().unwrap();
    assert!(matches!(
        prepared.publish(),
        Err(DomError::StyleReceiptMismatch)
    ));
    assert_eq!(lease.snapshot().stamp(), prefix.stamp());
    assert_eq!(
        lease.snapshot().incomplete_source_operation(),
        Some(read.operation())
    );
    let mut batch = root.batch(CssomEditGuard::from_snapshot(&changed)).unwrap();
    assert!(
        batch
            .set_sheet_list_membership(changed.sheet_list(), &[])
            .unwrap()
    );
    root.commit(&mut Some(batch)).unwrap();
    let restored = root.snapshot().unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(again)) if again == handoff
    ));

    let job = lease
        .prepare_fullscreen_exit_handoff(&handoff, &source, &cancel)
        .unwrap()
        .publish()
        .unwrap();
    assert_eq!(job.id(), handoff.invocation());
    assert_eq!(root.snapshot().unwrap().revision(), restored.revision());
    assert_eq!(lease.snapshot().node(fullscreen).unwrap().parent(), None);
    let DomSourceProgress::Request(DomSourceRequest::Style(insertion)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the later Style insertion must ask for its own CSP decision");
    };
    assert_eq!(insertion.source().operation(), read.operation());
    assert_eq!(insertion.source().element(), style);
    assert_eq!(
        insertion.action(),
        crate::DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
    );
    let denial = crate::DomStyleInlineCspObservation::evaluated(
        insertion.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        1,
        false,
    );
    lease
        .accept_inline_csp(&insertion, &denial, &cancel)
        .unwrap();
    let DomSourceProgress::Complete(completion) = lease.advance(&cancel).unwrap() else {
        panic!("the original source tail must complete after the exact handoff");
    };
    assert_eq!(completion.operation(), read.operation());
    assert_eq!(lease.snapshot().incomplete_source_operation(), None);
}

#[test]
fn replacement_closes_original_style_removal_after_svg_refusal() {
    use surgeist_html::elements::style::HtmlStyleUpdateAction;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let initial = dom.snapshot();
    let html = initial.node(document.node()).unwrap().children()[0];
    let parent = initial.node(html).unwrap().children()[1];
    let (root, sheet, before_cssom) =
        style_resource(&mut dom, document, parent, document.node(), false);
    let old = dom.snapshot().node(parent).unwrap().children()[0];
    let suffix = style_svg_suffix(&mut dom, document, false);
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    let before = dom.snapshot();
    let old_association = before
        .node(old)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert_eq!(old_association.id(), &sheet);
    assert_eq!(old_association.revision(), before_cssom.revision());
    assert_eq!(before.node(old).unwrap().parent(), Some(parent));
    let (read, prefix, removed) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_child(suffix.wrapper, parent, old),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(remove)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("replacement must first request removal of the old attached sheet");
        };
        assert_eq!(
            remove.action(),
            crate::DomStyleAction::Update(HtmlStyleUpdateAction::RemoveAssociatedSheet)
        );
        assert_eq!(remove.source().element(), old);
        assert_eq!(remove.sheet(), Some(old_association));
        assert_eq!(remove.cssom_root(), Some(document.node()));
        let remove = lease
            .qualify_cssom(&remove, root.root(), &before_cssom)
            .unwrap();
        let mut batch = root
            .batch(CssomEditGuard::from_snapshot(&before_cssom))
            .unwrap();
        batch.remove_sheet(&sheet).unwrap();
        let commit = root.commit(&mut Some(batch)).unwrap();
        let removed = root.snapshot().unwrap();
        assert_ne!(removed.revision(), before_cssom.revision());
        assert!(removed.sheets().is_empty());
        let detached_sheet = removed.sheet(&sheet).unwrap();
        assert!(detached_sheet.inputs().owner.is_none());
        assert!(detached_sheet.owner_rule().is_none());
        assert!(
            lease
                .accept_cssom_publication(&remove, commit, removed.clone(), None)
                .unwrap()
                .is_none()
        );
        let read = require_svg_fixture_script(&mut lease, &suffix, document, &cancel);
        let prefix = lease.snapshot();
        assert_eq!(prefix.node(old).unwrap().parent(), None);
        assert_eq!(prefix.node(suffix.wrapper).unwrap().parent(), None);
        assert_eq!(
            prefix.node(old).unwrap().as_style().unwrap().state(),
            before.node(old).unwrap().as_style().unwrap().state()
        );
        (read, prefix, removed)
    };
    let closed =
        resume_svg_fixture_failure(&mut dom, &binding, registration, read, &prefix, &cancel);
    suffix.check_private_scripts(&closed);
    assert_eq!(closed.node(old).unwrap().parent(), None);
    assert_eq!(closed.node(suffix.wrapper).unwrap().parent(), None);
    let current = root.snapshot().unwrap();
    assert_eq!(current.revision(), removed.revision());
    assert_eq!(current.sheets(), removed.sheets());
    assert!(current.sheet(&sheet).unwrap().inputs().owner.is_none());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "the original CSSOM removal must settle after SVG refusal: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    assert!(
        closed
            .node(old)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [
            crate::DomChange::ElementState(old),
            crate::DomChange::SourceOperation
        ]
    );
}

#[test]
fn accepted_inline_csp_denial_closes_marker_after_svg_refusal() {
    use surgeist_html::elements::style::HtmlStyleUpdateAction;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let initial = dom.snapshot();
    let html = initial.node(document.node()).unwrap().children()[0];
    let parent = initial.node(html).unwrap().children()[1];
    let suffix = style_svg_suffix(&mut dom, document, true);
    let style = suffix.style.unwrap();
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    let before = dom.snapshot();
    let old_style = before.node(style).unwrap().as_style().unwrap().state();
    let (read, prefix) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(suffix.wrapper, parent, None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the first inserted Style must reach its actual CSP request");
        };
        assert_eq!(
            policy.action(),
            crate::DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
        );
        assert_eq!(policy.source().element(), style);
        let denial = crate::DomStyleInlineCspObservation::evaluated(
            policy.source(),
            Arc::new(vec!["style-src 'none'".into()]),
            1,
            false,
        );
        lease.accept_inline_csp(&policy, &denial, &cancel).unwrap();
        let read = require_svg_fixture_script(&mut lease, &suffix, document, &cancel);
        let prefix = lease.snapshot();
        assert_eq!(prefix.node(suffix.wrapper).unwrap().parent(), Some(parent));
        assert_eq!(
            prefix.node(style).unwrap().as_style().unwrap().state(),
            old_style
        );
        (read, prefix)
    };
    let closed =
        resume_svg_fixture_failure(&mut dom, &binding, registration, read, &prefix, &cancel);
    suffix.check_private_scripts(&closed);
    assert_eq!(closed.node(suffix.wrapper).unwrap().parent(), Some(parent));
    assert_eq!(
        closed.node(style).unwrap().as_style().unwrap().state(),
        old_style
    );
    assert!(!dom.has_style_cssom_roots());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "the accepted CSP denial must allow the original marker to settle after SVG refusal: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    assert!(
        closed
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [crate::DomChange::SourceOperation]
    );
}

struct StyleSvgSuffix {
    wrapper: NodeId,
    style: Option<NodeId>,
    script: NodeId,
    svg: NodeId,
}

impl StyleSvgSuffix {
    fn check_private_scripts(&self, snapshot: &crate::DomSnapshot) {
        let children = snapshot.node(self.wrapper).unwrap().children();
        if let Some(style) = self.style {
            assert_eq!(children, [style, self.script, self.svg]);
        } else {
            assert_eq!(children, [self.script, self.svg]);
        }
        let html = snapshot
            .node(self.script)
            .unwrap()
            .as_html_script()
            .unwrap();
        assert!(!html.flags().already_started());
        assert_eq!(html.script_type(), None);
        assert_eq!(html.preparation_document(), None);
        let svg = snapshot.node(self.svg).unwrap().as_svg_script().unwrap();
        assert_eq!(svg.origin(), crate::DomSvgScriptOrigin::Native);
        assert!(!svg.flags().already_processed());
    }
}

fn style_svg_suffix(dom: &mut Dom, document: DocumentId, with_style: bool) -> StyleSvgSuffix {
    use surgeist_html::values::Namespace;

    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let wrapper = batch.create_element_ns(document, Namespace::html(), "div");
    let style = with_style.then(|| {
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        let css = batch.create_text(document, "div { color: red }");
        batch.insert_before(&css, &style, None);
        batch.insert_before(&style, &wrapper, None);
        style
    });
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let text = batch.create_text(document, "globalThis.ran = true");
    batch.insert_before(&text, &script, None);
    batch.insert_before(&script, &wrapper, None);
    let svg = batch.create_element_ns(&detached, Namespace::svg(), "script");
    batch.insert_before(&svg, &wrapper, None);
    let result = dom.apply(batch, &cancel()).unwrap();
    let resolve = |ticket| result.created(ticket).unwrap().unwrap();
    StyleSvgSuffix {
        wrapper: resolve(&wrapper),
        style: style.as_ref().map(resolve),
        script: resolve(&script),
        svg: resolve(&svg),
    }
}

fn attach_svg_cleanup_fixture(
    dom: &mut Dom,
    source: &DomHostedSource,
    document: DocumentId,
) -> (
    crate::DomDocumentScriptBinding,
    crate::DomDocumentScriptRegistration,
) {
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};

    let snapshot = dom.snapshot();
    let navigable = snapshot.document(document).unwrap().navigable().unwrap();
    let global = snapshot.hosted_navigable(navigable).unwrap().global();
    let backing = source
        .retain_runtime_payload(
            dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    source
        .attach_document_script_host(
            dom,
            global,
            document,
            crate::DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel(),
        )
        .unwrap()
}

fn require_svg_fixture_script(
    lease: &mut crate::DomSourceLease<'_>,
    suffix: &StyleSvgSuffix,
    document: DocumentId,
    cancel: &AtomicBool,
) -> crate::DomSourceScriptRequest {
    let DomSourceProgress::Request(DomSourceRequest::Script(read)) = lease.advance(cancel).unwrap()
    else {
        panic!(
            "the accepted Style action must be followed by the original Script settings request"
        );
    };
    assert_eq!(read.operation(), lease.operation());
    assert_eq!(read.step(), crate::DomSourceScriptStep::ReadScripting);
    assert_eq!(read.subject(), Some(suffix.script));
    assert_eq!(read.node_document(), Some(document));
    let prefix = lease.snapshot();
    assert_eq!(read.basis(), prefix.stamp());
    assert_eq!(prefix.incomplete_source_operation(), Some(read.operation()));
    assert!(lease.status().has_published());
    suffix.check_private_scripts(&prefix);
    assert!(
        matches!(lease.advance(cancel).unwrap(), DomSourceProgress::Request(DomSourceRequest::Script(again)) if again == read)
    );
    read
}

fn resume_svg_fixture_failure(
    dom: &mut Dom,
    binding: &crate::DomDocumentScriptBinding,
    registration: crate::DomDocumentScriptRegistration,
    read: crate::DomSourceScriptRequest,
    prefix: &crate::DomSnapshot,
    cancel: &AtomicBool,
) -> crate::DomSnapshot {
    assert_eq!(read.relevant_global(), Some(binding.global));
    drop(registration);
    dom.release_document_script_registration(binding);
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    let mut lease = dom
        .resume_source_edit(read.operation(), source_limits(), cancel)
        .unwrap();
    let refusal = lease.advance(cancel);
    assert!(
        matches!(refusal, Err(DomError::SvgScriptProcessingUnavailable)),
        "the original Disabled read must reach actual SVG refusal: {refusal:?}"
    );
    drop(lease);
    dom.snapshot()
}

#[test]
fn configured_inline_source_closes_empty_profile_after_svg_refusal() {
    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let initial = dom.snapshot();
    let html = initial.node(document.node()).unwrap().children()[0];
    let parent = initial.node(html).unwrap().children()[1];
    let suffix = style_svg_suffix(&mut dom, document, true);
    let style = suffix.style.unwrap();
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    dom.configure_inline_styles(
        crate::DomInlineStyleLimits::new(
            surgeist_cssom::CssomLimits::default(),
            32,
            64_000_000,
            8_000_000,
            4_000_000,
        )
        .unwrap(),
        &cancel,
    )
    .unwrap();
    let before = dom.snapshot();
    let old_style = before.node(style).unwrap().as_style().unwrap().state();
    let context = crate::DomInlineStyleSourceContext::new(before.stamp(), vec![]).unwrap();
    let (read, prefix) = {
        let mut lease = dom
            .begin_source_edit_with_inline_styles(
                DomSourceEdit::insert_before(suffix.wrapper, parent, None),
                context,
                source_limits(),
                crate::DomInlineStyleRequestLimits {
                    input: CssUtf16InputLimits::default(),
                    declaration: Default::default(),
                },
                &cancel,
            )
            .unwrap();
        let advance = |lease: &mut crate::DomSourceLease<'_>| {
            let (progress, inline) = lease
                .advance_with_inline_styles(no_svg_fixture_inline_inputs, &cancel)
                .unwrap()
                .into_parts();
            assert!(inline.is_none());
            progress
        };
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) = advance(&mut lease)
        else {
            panic!("configured inline source must publish its actual Style CSP request");
        };
        assert_eq!(policy.source().element(), style);
        let denial = crate::DomStyleInlineCspObservation::evaluated(
            policy.source(),
            Arc::new(vec!["style-src 'none'".into()]),
            1,
            false,
        );
        lease.accept_inline_csp(&policy, &denial, &cancel).unwrap();
        let read = profile_svg_fixture_script(advance(&mut lease), &lease, &suffix, document);
        assert!(
            matches!(advance(&mut lease), DomSourceProgress::Request(DomSourceRequest::Script(again)) if again == read)
        );
        (read, lease.snapshot())
    };
    assert_eq!(prefix.node(suffix.wrapper).unwrap().parent(), Some(parent));
    assert_eq!(
        prefix.node(style).unwrap().as_style().unwrap().state(),
        old_style
    );
    assert_eq!(read.relevant_global(), Some(binding.global));
    drop(registration);
    dom.release_document_script_registration(&binding);
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    let mut lease = dom
        .resume_source_edit(read.operation(), source_limits(), &cancel)
        .unwrap();
    match lease.advance_with_inline_styles(no_svg_fixture_inline_inputs, &cancel) {
        Err(crate::DomInlineStyleError::Dom(DomError::SvgScriptProcessingUnavailable)) => {}
        Err(error) => panic!("expected the original SVG refusal, got {error:?}"),
        Ok(_) => panic!("the actual SVG requirement must refuse"),
    }
    drop(lease);
    let closed = dom.snapshot();
    suffix.check_private_scripts(&closed);
    assert_eq!(closed.node(suffix.wrapper).unwrap().parent(), Some(parent));
    assert_eq!(
        closed.node(style).unwrap().as_style().unwrap().state(),
        old_style
    );
    assert!(!dom.has_style_cssom_roots());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "the empty configured inline profile must release the original marker: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [crate::DomChange::SourceOperation]
    );
}

#[test]
fn empty_meta_source_closes_accepted_cssom_sheet_after_svg_refusal() {
    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let initial = dom.snapshot();
    let html = initial.node(document.node()).unwrap().children()[0];
    let parent = initial.node(html).unwrap().children()[1];
    let suffix = style_svg_suffix(&mut dom, document, true);
    let style = suffix.style.unwrap();
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    let root = DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let caps = [&root];
    let empty = root.snapshot().unwrap();
    let limits = crate::DomMetaAtomicLimits::new(64_000_000, 1_000_000, 4_000_000, 256).unwrap();
    let before = dom.snapshot();
    let old_style = before.node(style).unwrap().as_style().unwrap().state();
    let (read, prefix, sheet, applied, mut notification) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(suffix.wrapper, parent, None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let advance = |lease: &mut crate::DomSourceLease<'_>| {
            let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
            let crate::DomMetaSourceStep::Progress(progress) =
                hold.prepare(&caps, None, ContextFree, &cancel).unwrap()
            else {
                panic!("an empty Meta recording must not produce a Meta boundary");
            };
            let (source, inline) = progress.into_parts();
            assert!(inline.is_none());
            source
        };
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) = advance(&mut lease)
        else {
            panic!("Meta-configured ordinary source must reach its actual CSP request");
        };
        assert_eq!(policy.source().element(), style);
        lease
            .accept_inline_csp(
                &policy,
                &crate::DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) = advance(&mut lease)
        else {
            panic!("accepted Style CSP must reach its CSSOM creation");
        };
        let (sheet, notification) = publish_style_sheet(&mut lease, &root, create);
        let applied = root.snapshot().unwrap();
        assert_ne!(applied.revision(), empty.revision());
        assert_eq!(applied.sheets(), std::slice::from_ref(&sheet));
        let read = profile_svg_fixture_script(advance(&mut lease), &lease, &suffix, document);
        assert!(
            matches!(advance(&mut lease), DomSourceProgress::Request(DomSourceRequest::Script(again)) if again == read)
        );
        assert_eq!(
            notification.origin(),
            &crate::DomStyleNotificationOrigin::NativeSource(read.operation())
        );
        (read, lease.snapshot(), sheet, applied, notification)
    };
    assert_eq!(prefix.node(suffix.wrapper).unwrap().parent(), Some(parent));
    assert_eq!(
        prefix.node(style).unwrap().as_style().unwrap().state(),
        old_style
    );
    assert_eq!(read.relevant_global(), Some(binding.global));
    drop(registration);
    dom.release_document_script_registration(&binding);
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    {
        let mut lease = dom
            .resume_source_edit(read.operation(), source_limits(), &cancel)
            .unwrap();
        {
            let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
            assert!(
                matches!(hold.prepare(&[], None, ContextFree, &cancel), Err(crate::DomMetaAtomicError::MissingOwner { root }) if root == document.node())
            );
        }
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        assert_eq!(
            lease.status().phase(),
            crate::DomSourcePhase::ScriptPreparation
        );
        let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
        match hold.prepare(&caps, None, ContextFree, &cancel) {
            Err(crate::DomMetaAtomicError::Dom(DomError::SvgScriptProcessingUnavailable)) => {}
            Err(error) => panic!("expected the original SVG refusal, got {error:?}"),
            Ok(_) => panic!("the actual SVG requirement must refuse"),
        };
    }
    let closed = dom.snapshot();
    suffix.check_private_scripts(&closed);
    assert_eq!(closed.node(suffix.wrapper).unwrap().parent(), Some(parent));
    let current = root.snapshot().unwrap();
    assert_eq!(current.revision(), applied.revision());
    assert_eq!(current.sheets(), applied.sheets());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "empty Meta must settle the accepted CSSOM receipt and original marker: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    let associated = closed
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert_eq!(associated.id(), &sheet);
    assert_eq!(associated.revision(), applied.revision());
    assert_eq!(associated.association(), notification.association());
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [
            crate::DomChange::ElementState(style),
            crate::DomChange::SourceOperation
        ]
    );
    assert!(matches!(
        dom.finish_style_notification(
            &mut notification,
            crate::DomStyleNotificationDelivery::Native,
            1,
            &cancel
        )
        .unwrap(),
        crate::DomStyleNotificationCompletion::Complete(_)
    ));
}

fn no_svg_fixture_inline_inputs(
    _: &[surgeist_cssom::CssomDeclarationCandidate],
    _: &surgeist_cssom::CssomContext,
) -> Result<surgeist_cssom::CssomAdmission, surgeist_cssom::CssomError> {
    panic!("this fixture has no inline declaration inputs")
}

fn profile_svg_fixture_script(
    progress: DomSourceProgress,
    lease: &crate::DomSourceLease<'_>,
    suffix: &StyleSvgSuffix,
    document: DocumentId,
) -> crate::DomSourceScriptRequest {
    let DomSourceProgress::Request(DomSourceRequest::Script(read)) = progress else {
        panic!("actual Style receipt must be followed by the original Script settings request");
    };
    assert_eq!(read.operation(), lease.operation());
    assert_eq!(read.step(), crate::DomSourceScriptStep::ReadScripting);
    assert_eq!(read.subject(), Some(suffix.script));
    assert_eq!(read.node_document(), Some(document));
    let prefix = lease.snapshot();
    assert_eq!(read.basis(), prefix.stamp());
    assert_eq!(prefix.incomplete_source_operation(), Some(read.operation()));
    assert!(lease.status().has_published());
    suffix.check_private_scripts(&prefix);
    read
}

#[test]
fn stopped_registration_does_not_accept_a_prior_script_settings_receipt() {
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, document) = constructor();
    let snapshot = dom.snapshot();
    let navigable = snapshot.document(document).unwrap().navigable().unwrap();
    let global = snapshot.hosted_navigable(navigable).unwrap().global();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];

    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let text = batch.create_text(document, "globalThis.ran = true");
    batch.insert_before(&text, &script, None);
    let script = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&script)
        .unwrap()
        .unwrap();
    let facts = crate::DomDocumentScriptingFacts {
        engine: HtmlScriptingEngine::Supported,
        policy: HtmlScriptingPolicy {
            user_allows: true,
            automation_allows: true,
        },
    };
    let backing = source
        .retain_runtime_payload(
            &mut dom,
            crate::DomDocumentScriptRegistration::storage_bytes(),
            &cancel,
        )
        .unwrap();
    let (binding, registration) = source
        .attach_document_script_host(&mut dom, global, document, facts, backing, &cancel)
        .unwrap();
    let (request, observation) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(script, body, None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Script(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the original Script must park at its live read");
        };
        let observation = lease
            .observe_script_settings(request, &source, &binding, facts, &cancel)
            .unwrap();
        assert!(observation.enabled());
        (request, observation)
    };

    drop(registration);
    dom.release_document_script_registration(&binding);
    assert_eq!(dom.snapshot().stamp(), request.basis());
    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel)
        .unwrap();
    assert_eq!(
        lease.accept_script_settings(observation, &cancel),
        Err(DomError::InvalidSourceOperation)
    );
    assert_eq!(lease.operation(), request.operation());
    assert_eq!(lease.snapshot().stamp(), request.basis());
    assert_eq!(lease.snapshot().node(script).unwrap().parent(), None);
    lease.source_script_step(request, &cancel).unwrap();
    drop(source);
}

#[test]
fn rejected_svg_suffix_does_not_publish_private_meta_recording() {
    use surgeist_html::values::Namespace;

    let cancel = cancel();
    let (mut dom, source, fixture) = svg_removal_fixture();
    let SvgRemovalFixture {
        document,
        parent,
        old,
        root,
        suffix,
        ..
    } = &fixture;
    let (document, parent, old) = (*document, *parent, *old);
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "rejected preference");
    batch.insert_before(&meta, suffix.wrapper, Some(suffix.script.into()));
    let meta = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&meta)
        .unwrap()
        .unwrap();
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    let before = dom.snapshot();
    let old_style = before.node(old).unwrap().as_style().unwrap().state();
    let caps = [root];
    let limits = crate::DomMetaAtomicLimits::new(64_000_000, 1_000_000, 4_000_000, 256).unwrap();
    let advance = |lease: &mut crate::DomSourceLease<'_>| {
        let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
        let crate::DomMetaSourceStep::Progress(progress) =
            hold.prepare(&caps, None, ContextFree, &cancel).unwrap()
        else {
            panic!("a private rejected suffix must not expose a Meta producer preparation");
        };
        let (progress, inline) = progress.into_parts();
        assert!(inline.is_none());
        progress
    };
    let (read, prefix, removed) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_child(suffix.wrapper, parent, old),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let progress = advance(&mut lease);
        let removed = fixture.accept_removal(&mut lease, progress);
        let DomSourceProgress::Request(DomSourceRequest::Script(read)) = advance(&mut lease) else {
            panic!("the privately recorded Meta must precede the original HTML settings read");
        };
        assert_eq!(read.operation(), lease.operation());
        assert_eq!(read.step(), crate::DomSourceScriptStep::ReadScripting);
        assert_eq!(read.subject(), Some(suffix.script));
        assert_eq!(read.node_document(), Some(document));
        assert_eq!(read.relevant_global(), Some(binding.global));
        let prefix = lease.snapshot();
        assert_eq!(prefix.stamp(), read.basis());
        assert_eq!(prefix.incomplete_source_operation(), Some(read.operation()));
        assert_eq!(prefix.node(old).unwrap().parent(), None);
        assert_eq!(prefix.node(suffix.wrapper).unwrap().parent(), None);
        assert_eq!(
            prefix.node(old).unwrap().as_style().unwrap().state(),
            old_style
        );
        // The ordinary advance detects the real retained Meta occurrence before
        // clearing its recording or consuming the parked Script settings.
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::Meta(crate::DomMetaError::DefaultStyleUnsupported))
        ));
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        assert!(
            matches!(advance(&mut lease), DomSourceProgress::Request(DomSourceRequest::Script(again)) if again == read)
        );
        (read, prefix, removed)
    };
    drop(registration);
    dom.release_document_script_registration(&binding);
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    let spent = dom.changes_since(before.stamp()).unwrap().changes().len();
    let with_outputs = |outputs| {
        let limits = source_limits();
        DomSourceLimits::new(
            limits.retained_bytes(),
            limits.retained_string_code_units(),
            limits.work(),
            limits.phase_publications(),
            outputs,
            limits.published_native_identities(),
            limits.nested_operations(),
        )
    };
    {
        let mut lease = dom
            .resume_source_edit(read.operation(), with_outputs(spent + 1), &cancel)
            .unwrap();
        let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
        match hold.prepare(&caps, None, ContextFree, &cancel) {
            Err(crate::DomMetaAtomicError::Dom(DomError::SourceOutputLimit)) => {}
            Err(error) => {
                panic!("one missing closing output must retain the selected SVG failure: {error:?}")
            }
            Ok(_) => panic!("a one-output closing allowance must refuse"),
        };
    }
    let cleanup = dom.pending_source_edit().unwrap();
    assert_eq!(cleanup.operation(), read.operation());
    assert_eq!(cleanup.phase(), crate::DomSourcePhase::PendingCleanup);
    assert_eq!(
        cleanup.cleanup_cause(),
        Some(crate::DomSourceCleanupCause::SvgScriptProcessingUnavailable)
    );
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    assert_eq!(root.snapshot().unwrap().revision(), removed.revision());
    {
        let mut lease = dom
            .resume_source_edit(read.operation(), with_outputs(spent + 2), &cancel)
            .unwrap();
        let mut hold = lease.begin_meta_runtime(limits, &cancel).unwrap();
        match hold.prepare(&caps, None, ContextFree, &cancel) {
            Err(crate::DomMetaAtomicError::Dom(DomError::SvgScriptProcessingUnavailable)) => {}
            Err(error) => panic!("expected the original SVG refusal, got {error:?}"),
            Ok(_) => {
                panic!("the actual SVG requirement must refuse before Meta producer preparation")
            }
        };
    }
    let closed = dom.snapshot();
    assert_eq!(closed.node(old).unwrap().parent(), None);
    assert_eq!(closed.node(suffix.wrapper).unwrap().parent(), None);
    assert_eq!(
        closed.node(suffix.wrapper).unwrap().children(),
        [meta, suffix.script, suffix.svg]
    );
    let script = closed
        .node(suffix.script)
        .unwrap()
        .as_html_script()
        .unwrap();
    assert!(!script.flags().already_started());
    assert_eq!(script.preparation_document(), None);
    assert!(
        !closed
            .node(suffix.svg)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
    let current = root.snapshot().unwrap();
    assert_eq!(current.revision(), removed.revision());
    assert_eq!(current.sheets(), removed.sheets());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "the private Meta input must not strand the original Source after SVG refusal: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    assert!(
        closed
            .node(old)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [
            crate::DomChange::ElementState(old),
            crate::DomChange::SourceOperation
        ]
    );
}

#[test]
fn rejected_svg_suffix_does_not_publish_private_inline_recording() {
    let cancel = cancel();
    let (mut dom, source, fixture) = svg_removal_fixture();
    let SvgRemovalFixture {
        document,
        parent,
        old,
        root,
        suffix,
        ..
    } = &fixture;
    let (document, parent, old) = (*document, *parent, *old);
    let (binding, registration) = attach_svg_cleanup_fixture(&mut dom, &source, document);
    dom.configure_inline_styles(
        crate::DomInlineStyleLimits::new(
            surgeist_cssom::CssomLimits::default(),
            32,
            64_000_000,
            8_000_000,
            4_000_000,
        )
        .unwrap(),
        &cancel,
    )
    .unwrap();
    let request_limits = crate::DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: Default::default(),
    };
    let parser = CssParserContext::new(surgeist_css::CssParserMode::Quirks);
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "private SVG suffix".into(),
        revision: 1,
    };
    let inputs = crate::DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        parser,
        version.clone(),
        Default::default(),
    )
    .unwrap();
    let properties = dom
        .inline_style(
            suffix.wrapper,
            &inputs,
            request_limits,
            no_svg_fixture_inline_inputs,
            &cancel,
        )
        .unwrap();
    let inline_before = dom.capture_inline_style(suffix.wrapper).unwrap().unwrap();
    assert_eq!(inline_before.tree_scope().root(), suffix.wrapper);
    assert_eq!(inline_before.properties().handle(), &properties);
    let before = dom.snapshot();
    let old_style = before.node(old).unwrap().as_style().unwrap().state();
    let profile = crate::DomInlineStyleProfile::new(
        suffix.wrapper,
        document,
        parser,
        version,
        Default::default(),
    )
    .unwrap();
    let context = crate::DomInlineStyleSourceContext::new(before.stamp(), vec![profile]).unwrap();
    let advance = |lease: &mut crate::DomSourceLease<'_>| {
        let (source, inline) = lease
            .advance_with_inline_styles(no_svg_fixture_inline_inputs, &cancel)
            .unwrap()
            .into_parts();
        assert!(inline.is_none());
        source
    };
    let (read, prefix, removed) = {
        let mut lease = dom
            .begin_source_edit_with_inline_styles(
                DomSourceEdit::replace_child(suffix.wrapper, parent, old),
                context,
                source_limits(),
                request_limits,
                &cancel,
            )
            .unwrap();
        let progress = advance(&mut lease);
        let removed = fixture.accept_removal(&mut lease, progress);
        assert!(!lease.has_private_inline_inputs());
        let read = profile_svg_fixture_script(advance(&mut lease), &lease, suffix, document);
        assert!(lease.has_private_inline_inputs());
        assert!(
            matches!(advance(&mut lease), DomSourceProgress::Request(DomSourceRequest::Script(again)) if again == read)
        );
        let prefix = lease.snapshot();
        assert_eq!(prefix.node(old).unwrap().parent(), None);
        assert_eq!(prefix.node(suffix.wrapper).unwrap().parent(), None);
        assert_eq!(
            prefix.node(old).unwrap().as_style().unwrap().state(),
            old_style
        );
        (read, prefix, removed)
    };
    assert_eq!(read.relevant_global(), Some(binding.global));
    drop(registration);
    dom.release_document_script_registration(&binding);
    assert_eq!(dom.snapshot().stamp(), prefix.stamp());
    let mut lease = dom
        .resume_source_edit(read.operation(), source_limits(), &cancel)
        .unwrap();
    match lease.advance_with_inline_styles(no_svg_fixture_inline_inputs, &cancel) {
        Err(crate::DomInlineStyleError::Dom(DomError::SvgScriptProcessingUnavailable)) => {}
        Err(error) => panic!("expected the original SVG refusal, got {error:?}"),
        Ok(_) => panic!("the actual SVG requirement must refuse before an inline publication"),
    }
    drop(lease);
    let closed = dom.snapshot();
    suffix.check_private_scripts(&closed);
    assert_eq!(closed.node(old).unwrap().parent(), None);
    assert_eq!(closed.node(suffix.wrapper).unwrap().parent(), None);
    assert_eq!(root.snapshot().unwrap().revision(), removed.revision());
    let pending = dom.pending_source_edit();
    assert!(
        pending.is_none(),
        "private inline input must not strand the original Source after SVG refusal: {pending:?}"
    );
    assert_eq!(closed.incomplete_source_operation(), None);
    assert!(
        closed
            .node(old)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    let inline_after = dom.capture_inline_style(suffix.wrapper).unwrap().unwrap();
    assert_eq!(inline_after.tree_scope().root(), suffix.wrapper);
    assert_eq!(inline_after.properties().handle(), &properties);
    assert_eq!(
        inline_after.properties().snapshot().revision(),
        inline_before.properties().snapshot().revision()
    );
    assert_eq!(
        dom.changes_since(prefix.stamp()).unwrap().changes(),
        [
            crate::DomChange::ElementState(old),
            crate::DomChange::SourceOperation
        ]
    );
}

struct SvgRemovalFixture {
    document: DocumentId,
    parent: NodeId,
    old: NodeId,
    root: DomStyleCssomRoot,
    sheet: surgeist_cssom::CssomSheetId,
    before_cssom: CssomSnapshot,
    suffix: StyleSvgSuffix,
}

fn svg_removal_fixture() -> (Dom, DomHostedSource, SvgRemovalFixture) {
    let (mut dom, source, document) = constructor_with_meta_work(1_000_000);
    let initial = dom.snapshot();
    let html = initial.node(document.node()).unwrap().children()[0];
    let parent = initial.node(html).unwrap().children()[1];
    let (root, sheet, before_cssom) =
        style_resource(&mut dom, document, parent, document.node(), false);
    let old = dom.snapshot().node(parent).unwrap().children()[0];
    let suffix = style_svg_suffix(&mut dom, document, false);
    (
        dom,
        source,
        SvgRemovalFixture {
            document,
            parent,
            old,
            root,
            sheet,
            before_cssom,
            suffix,
        },
    )
}

impl SvgRemovalFixture {
    fn accept_removal(
        &self,
        lease: &mut crate::DomSourceLease<'_>,
        progress: DomSourceProgress,
    ) -> CssomSnapshot {
        use surgeist_html::elements::style::HtmlStyleUpdateAction;

        let DomSourceProgress::Request(DomSourceRequest::Style(remove)) = progress else {
            panic!("the replacement must first request its original CSSOM removal");
        };
        assert_eq!(
            remove.action(),
            crate::DomStyleAction::Update(HtmlStyleUpdateAction::RemoveAssociatedSheet)
        );
        assert_eq!(remove.source().element(), self.old);
        assert_eq!(remove.sheet().unwrap().id(), &self.sheet);
        let remove = lease
            .qualify_cssom(&remove, self.root.root(), &self.before_cssom)
            .unwrap();
        let mut batch = self
            .root
            .batch(CssomEditGuard::from_snapshot(&self.before_cssom))
            .unwrap();
        batch.remove_sheet(&self.sheet).unwrap();
        let commit = self.root.commit(&mut Some(batch)).unwrap();
        let removed = self.root.snapshot().unwrap();
        assert_ne!(removed.revision(), self.before_cssom.revision());
        assert!(removed.sheets().is_empty());
        assert!(
            lease
                .accept_cssom_publication(&remove, commit, removed.clone(), None)
                .unwrap()
                .is_none()
        );
        removed
    }
}
