//! One canonical subtree edit retains ordered Style and native child hooks.
//! Ordinary batches remain atomic; Source publishes coherent phases (#1219).
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_cssom::{CssomSnapshot, CssomStore};
use surgeist_dom::*;
use surgeist_html::HtmlStyleUpdateAction;
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseParseObservation, HtmlBaseParseOutcome,
    HtmlBaseProducerId, HtmlBaseUrlRecord,
};
use surgeist_runtime::*;

const CSS: &str = "p { color: red }";
const SRC: &str = "http://[";
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 32_000_000, 256, 8192, 8192, 4)
}
fn hosted_limits(registries: usize, tasks: usize) -> DomHostedRuntimeLimits {
    DomHostedRuntimeLimits {
        canonical: DomHostedLimits::new(
            DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, registries),
            32_000_000,
            32_000_000,
            8_000_000,
        ),
        tasks,
        storage_bytes: 32_000_000,
        scratch_bytes: 32_000_000,
        work: 8_000_000,
    }
}
fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 32_000_000,
            work: 8_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 16,
            reports: 16,
            storage_bytes: 32_000_000,
            work: 8_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 16,
            storage_bytes: 32_000_000,
            work: 8_000_000,
        },
        hosted: hosted_limits(32, 32),
    }
}
struct Fixture {
    session: DomSession,
    top: DomHostedTopLevel,
    body: NodeId,
    closed: DocumentId,
    root: NodeId,
    iframe: NodeId,
    style: NodeId,
}
impl Fixture {
    fn new() -> Self {
        let mut session = DomSession::new(
            Dom::new(
                DomLimits::new(
                    512,
                    2_000_000,
                    2048,
                    2048,
                    2048,
                    64,
                    8_000_000,
                    surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                        .unwrap(),
                )
                .unwrap()
                .with_dispatch_custody(
                    DomDispatchCustodyLimits::new(32, 2048, 32_000_000, 32_000_000, 8_000_000)
                        .unwrap(),
                ),
            )
            .unwrap(),
            session_limits(),
        )
        .unwrap();
        let mut hosted = session.hosted_execution();
        let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
        let top = hosted
            .create_top_level(&mut session.dom(), host, &cancel())
            .unwrap();
        let snapshot = session.snapshot();
        let html = snapshot.node(top.document().node()).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        let mut styles = session.style_execution();
        let mut input = Some(DomStyleCssomRoot::new(
            top.document().node(),
            CssomStore::new(Default::default(), Default::default()).unwrap(),
        ));
        styles
            .bind_root(&session.dom(), "mixed-root".into(), &mut input, &cancel())
            .unwrap();
        assert!(input.is_none());
        // The closed Document is real constructor state, not a false activity
        // observation. Detached preparation reaches no hosted/Style boundary.
        let mut batch = DomBatch::new();
        let closed = batch.create_html_document(top.document(), None);
        let root = batch.create_element_ns(&closed, Namespace::html(), "div");
        let iframe = batch.create_element_ns(&closed, Namespace::html(), "iframe");
        batch.set_attribute(&iframe, "src", SRC);
        batch.set_attribute(
            &iframe,
            "allow",
            "fullscreen *; focus-without-user-activation *",
        );
        batch.insert_before(&iframe, &root, None);
        let style = batch.create_element_ns(&closed, Namespace::html(), "style");
        let text = batch.create_text(&closed, CSS);
        batch.insert_before(&text, &style, None);
        batch.insert_before(&style, &root, None);
        let commit = session.dom().apply(batch, &cancel()).unwrap();
        let closed = commit.document(&closed).unwrap();
        let root = commit.created(&root).unwrap().unwrap();
        let iframe = commit.created(&iframe).unwrap().unwrap();
        let style = commit.created(&style).unwrap().unwrap();
        let ready = session.snapshot();
        assert!(!ready.document(closed).unwrap().is_fully_active().unwrap());
        assert_eq!(ready.node(root).unwrap().parent(), None);
        assert_eq!(ready.node(root).unwrap().children(), [iframe, style]);
        assert_eq!(ready.node(iframe).unwrap().owner_document(), Some(closed));
        assert_eq!(
            ready
                .node(iframe)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
        assert!(
            ready
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .is_none()
        );
        assert!(
            styles
                .snapshot(top.document().node())
                .unwrap()
                .unwrap()
                .sheets()
                .is_empty()
        );
        Self {
            session,
            top,
            body,
            closed,
            root,
            iframe,
            style,
        }
    }
    fn cssom(&self) -> CssomSnapshot {
        self.session
            .style_execution()
            .snapshot(self.top.document().node())
            .unwrap()
            .unwrap()
    }
    fn start(&mut self, csp: &mut Vec<DomSourceOperationId>) -> DomHostedChildRequest {
        let style = self.style;
        let document = self.top.document();
        let mut policy = |source: &DomStyleSource| {
            assert_eq!(source.element(), style);
            assert_eq!(source.document(), document);
            assert_eq!(source.root(), document.node());
            assert_eq!(
                source.text().as_slice(),
                CSS.encode_utf16().collect::<Vec<_>>()
            );
            csp.push(source.operation());
            Some(DomStyleInlineCspObservation::complete_empty(source, 1))
        };
        // This permission is temporary. Its drop cannot erase the published
        // sheet, queued Load or original canonical child continuation.
        let mut source = self.session.source_execution();
        let result = source
            .apply(
                &mut self.session.dom(),
                DomSourceEdit::insert_before(self.root, self.body, None),
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        match result {
            DomSourceDriveProgress::Pending {
                request: DomSourceRequest::HostedChild(child),
                reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
            } => {
                assert_eq!(child.container(), self.iframe);
                assert_eq!(child.parent_document().document(), self.top.document());
                assert_eq!(child.parent_global(), self.top.global());
                child
            }
            other => panic!("Style must precede the genuine post-connection child wait: {other:?}"),
        }
    }
    fn resume_url(
        &mut self,
        operation: DomSourceOperationId,
        inputs: &UrlInputs,
        csp: &mut Vec<DomSourceOperationId>,
        stages: &mut Vec<DomSourceUrlStage>,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let mut policy = |source: &DomStyleSource| {
            csp.push(source.operation());
            Some(DomStyleInlineCspObservation::complete_empty(source, 1))
        };
        let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
            inputs.check(request);
            assert_eq!(request.operation(), operation);
            stages.push(request.stage());
            // URLfde3: an unclosed IPv6 host is a real parse Failure. HTML
            // falls back to parent origin/about:blank; this is not an aborted
            // Source command and supplies no invented URL or origin identity.
            Ok(Some(request.observe(inputs.failure(), None).unwrap()))
        };
        self.session.source_execution().resume(
            &mut self.session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut policy,
                url: Some(DomSourceUrlService {
                    revision: &inputs.revision,
                    observe: &mut observe,
                }),
            },
            &cancel(),
        )
    }
    fn resume_without_url(
        &mut self,
        operation: DomSourceOperationId,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        self.session.source_execution().resume(
            &mut self.session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_: &DomStyleSource| None,
                url: None,
            },
            cancel,
        )
    }
    fn detach(&mut self) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        self.session.source_execution().apply(
            &mut self.session.dom(),
            DomSourceEdit::detach(self.root),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_: &DomStyleSource| None,
                url: None,
            },
            &cancel(),
        )
    }
}
struct UrlInputs {
    document: DocumentId,
    base: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    revision: HtmlBaseFactsRevision,
    origin: DomOpaqueOrigin,
}
impl UrlInputs {
    fn new(fixture: &Fixture) -> Self {
        let snapshot = fixture.session.snapshot();
        let document = snapshot.document(fixture.top.document()).unwrap();
        let facts = document.base_document_facts().unwrap();
        let base = facts.document_url().unwrap().clone();
        assert_eq!(base.serialization(), &DocumentString::from("about:blank"));
        Self {
            document: fixture.top.document(),
            base,
            encoding: facts.encoding().unwrap().clone(),
            revision: HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 11, 12, 13),
            origin: document.metadata().origin().clone(),
        }
    }
    fn check(&self, request: &DomSourceUrlRequest) {
        assert_eq!(request.document().document(), self.document);
        assert_eq!(request.raw(), &DocumentString::from(SRC));
        assert_eq!(request.base_url(), &self.base);
        assert_eq!(request.encoding(), &self.encoding);
        assert_eq!(request.revision(), &self.revision);
        assert_eq!(
            request.target_origin(),
            match request.stage() {
                DomSourceUrlStage::IframeDeclaredOrigin => Some(&self.origin),
                DomSourceUrlStage::FrameIframeSrc => None,
            }
        );
    }
    fn failure(&self) -> HtmlBaseParseObservation {
        HtmlBaseParseObservation::new(
            SRC.into(),
            self.base.clone(),
            self.encoding.clone(),
            self.revision.clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap()
    }
}
fn complete(progress: DomSourceDriveProgress, operation: DomSourceOperationId) {
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("real mandatory tail must complete: {progress:?}")
    };
    assert_eq!(done.operation(), operation);
    assert!(!done.cancellation_requested());
}
fn sheet(snapshot: &DomSnapshot, style: NodeId) -> Option<DomStyleSheet> {
    snapshot
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .cloned()
}
fn child(snapshot: &DomSnapshot, iframe: NodeId) -> DomHostedNavigableId {
    snapshot
        .node(iframe)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap()
}
fn check_published_style(fixture: &Fixture, operation: DomSourceOperationId) -> CssomSnapshot {
    let snapshot = fixture.session.snapshot();
    assert_eq!(snapshot.incomplete_source_operation(), Some(operation));
    assert_eq!(
        snapshot.node(fixture.root).unwrap().parent(),
        Some(fixture.body)
    );
    assert_eq!(
        snapshot.node(fixture.style).unwrap().owner_document(),
        Some(fixture.top.document())
    );
    assert!(matches!(
        snapshot
            .node(fixture.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet(),
        Err(DomError::IncompleteSourceOperation)
    ));
    let current = fixture.cssom();
    assert_eq!(current.sheets().len(), 1);
    let id = &current.sheets()[0];
    let actual = current.sheet(id).unwrap();
    assert_eq!(actual.authored().rules().len(), 1);
    assert!(actual.diagnostics().is_empty());
    assert!(actual.inputs().owner.is_some());
    let styles = fixture.session.style_execution();
    assert_eq!(
        styles.pending_sheet(0).unwrap(),
        Some((fixture.top.document().node(), id.clone()))
    );
    assert_eq!(styles.pending_sheet(1).unwrap(), None);
    assert!(styles.processed_sheet(0).unwrap().is_some());
    assert!(styles.processed_sheet(1).unwrap().is_none());
    current
}
fn check_joined(fixture: &Fixture, original: &CssomSnapshot) -> (DomHostedNavigableId, DocumentId) {
    let snapshot = fixture.session.snapshot();
    assert_eq!(snapshot.incomplete_source_operation(), None);
    assert!(
        fixture
            .session
            .snapshot()
            .incomplete_hosted_operation()
            .is_none()
    );
    let associated = sheet(&snapshot, fixture.style).unwrap();
    assert_eq!(associated.id(), &original.sheets()[0]);
    assert_eq!(associated.root(), fixture.top.document().node());
    assert_eq!(fixture.cssom().revision(), original.revision());
    let id = child(&snapshot, fixture.iframe);
    let document = snapshot
        .hosted_navigable(id)
        .unwrap()
        .active_document()
        .unwrap();
    assert!(
        snapshot
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot.document(document).unwrap().metadata().origin(),
        snapshot
            .document(fixture.top.document())
            .unwrap()
            .metadata()
            .origin()
    );
    let policy = snapshot
        .document(document)
        .unwrap()
        .metadata()
        .initial_permissions_policy()
        .unwrap();
    assert_eq!(
        policy,
        DomInitialPermissionsPolicy::new(
            DomPermissionsPolicyValue::Enabled,
            DomPermissionsPolicyValue::Enabled
        )
    );
    (id, document)
}
fn drain_hosted_fixture_until_idle(fixture: &mut Fixture) -> Vec<DomHostedTaskProgress> {
    let mut hosted = fixture.session.hosted_execution();
    let mut observations = Vec::new();
    for _ in 0..256 {
        let progress = hosted
            .run_next(
                &mut fixture.session.dom(),
                fixture.top.navigable(),
                &cancel(),
            )
            .unwrap_or_else(|error| {
                panic!("native fixture work refused after {observations:?}: {error:?}")
            });
        // This fixture starts only its one child request/removal. A captured
        // History completion does not finish the retained destruction wrapper;
        // actual FIFO idle establishes that both owners have settled.
        if matches!(progress, DomHostedTaskProgress::Idle) {
            assert!(
                !observations.is_empty(),
                "actual queued work must not disappear"
            );
            assert!(
                observations
                    .iter()
                    .any(|step| matches!(step, DomHostedTaskProgress::Complete { .. })),
                "actual work must complete before idle: {observations:?}"
            );
            return observations;
        }
        observations.push(progress);
    }
    panic!("actual native fixture queue did not reach idle in 256 advances: {observations:?}")
}
struct Networking {
    styles: DomStyleRuntime,
    delivered: Vec<(DomEventType, NodeId)>,
}
impl DomStyleEventContext for Networking {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}
fn deliver_load(fixture: &mut Fixture) {
    let mut context = Networking {
        styles: fixture.session.style_execution(),
        delivered: Vec::new(),
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let style = fixture.style;
    let once = registry
        .add_listener(
            &fixture.session.dom(),
            style,
            DomEventType::Load,
            DomEventCallback::new(move |_, _, context: &mut Networking, event| {
                assert_eq!(event.event_type(), DomEventType::Load);
                assert_eq!(event.target(), Some(style));
                assert!(!event.cancelable());
                context.delivered.push((event.event_type(), style));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let limits = DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 8,
        failure_records: 8,
        callbacks: 8,
        work: 100_000,
    };
    let result = DomStyleRuntime::run_networking(
        &mut fixture.session.dom(),
        &mut registry,
        &mut context,
        limits,
        &cancel(),
    )
    .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = result else {
        panic!("actual queued stylesheet Load: {result:?}")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert_eq!(context.delivered, [(DomEventType::Load, style)]);
    assert!(!registry.remove_listener(once));
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut fixture.session.dom(),
            &mut registry,
            &mut context,
            limits,
            &cancel()
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(context.styles.pending_sheet(0).unwrap(), None);
}
fn removal_complete(fixture: &Fixture, id: DomHostedNavigableId, old_cssom: &CssomSnapshot) {
    let removed = fixture.session.snapshot();
    assert_eq!(removed.incomplete_source_operation(), None);
    assert_eq!(removed.node(fixture.root).unwrap().parent(), None);
    assert_eq!(
        removed
            .node(fixture.iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert!(removed.hosted_navigable(id).unwrap().is_retiring());
    assert!(sheet(&removed, fixture.style).is_none());
    let current = fixture.cssom();
    assert!(current.sheets().is_empty());
    assert!(
        current
            .sheet(&old_cssom.sheets()[0])
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    assert!(
        old_cssom
            .sheet(&old_cssom.sheets()[0])
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
}
fn settle_removal(fixture: &mut Fixture, id: DomHostedNavigableId, document: DocumentId) {
    let observations = drain_hosted_fixture_until_idle(fixture);
    assert_eq!(
        observations
            .iter()
            .filter(|step| matches!(step, DomHostedTaskProgress::Executed {
        document: actual, kind: DomHostedTaskKind::Destruction, ..
    } if *actual == document))
            .count(),
        1,
        "one real child destructor: {observations:?}"
    );
    let snapshot = fixture.session.snapshot();
    assert!(snapshot.hosted_navigable(id).unwrap().is_retired());
    assert!(
        !snapshot
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        snapshot
            .hosted_entry(fixture.top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
}
fn teardown(mut fixture: Fixture) {
    let mut hosted = fixture.session.hosted_execution();
    let styles = fixture.session.style_execution();
    assert!(fixture.session.dom().pending_source_edit().is_none());
    assert!(fixture.cssom().sheets().is_empty());
    assert_eq!(styles.pending_sheet(0).unwrap(), None);
    assert_eq!(styles.waiting_resources().unwrap(), 0);
    assert!(matches!(
        hosted
            .run_next(
                &mut fixture.session.dom(),
                fixture.top.navigable(),
                &cancel()
            )
            .unwrap(),
        DomHostedTaskProgress::Idle
    ));
    assert!(
        fixture
            .session
            .snapshot()
            .document(fixture.top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    // DOM's hosted cleanup guard rejects a still-bound Document CSSOM root,
    // even after its sheet and notifications are gone. There is no public
    // unbind. All started Source/child/history/Load work is settled, while the
    // original Top/Host and empty root remain live resources.
    let session = match fixture.session.try_shutdown() {
        DomShutdownOutcome::Pending { session, reason } => {
            assert_eq!(reason, DomShutdownReason::HostedLifecycle);
            session
        }
        other => panic!("live hosted resources and bound empty CSSOM root remain: {other:?}"),
    };
    // Terminal disposal abandons those live resources, without pretending that
    // a Global retirement or destruction callback was completed.
    drop(session);
    assert!(matches!(
        styles.snapshot(fixture.top.document().node()),
        Err(DomStyleRuntimeError::SessionUnavailable)
    ));
    // An independently live empty view cannot revive the disposed original
    // Hosted owner. This control creates no Host/Global/task in that session.
    let mut control = DomSession::new(
        Dom::new(
            DomLimits::new(
                1,
                1,
                0,
                0,
                0,
                0,
                0,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
        session_limits(),
    )
    .unwrap();
    assert!(matches!(
        hosted.settings(&control.dom(), fixture.top.global(), &cancel()),
        Err(DomHostedRuntimeError::OwnerDisposed)
    ));
    assert!(matches!(
        control.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

#[test]
fn mixed_insertion_keeps_published_stylesheet_through_child_url_wait_registry_refusal_and_exact_retry()
 {
    let mut fixture = Fixture::new();
    let before = fixture.session.snapshot();
    let before_cssom = fixture.cssom();
    let inputs = UrlInputs::new(&fixture);
    fixture
        .session
        .hosted_execution()
        .set_limits(&mut fixture.session.dom(), hosted_limits(1, 32), &cancel())
        .unwrap();
    let mut csp = Vec::new();
    let request = fixture.start(&mut csp);
    let operation = request.operation();
    assert_eq!(csp, [operation]);
    let original = check_published_style(&fixture, operation);
    let pending = fixture.session.snapshot();
    assert_eq!(
        pending
            .node(fixture.iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    let mut competing = DomBatch::new();
    competing.set_attribute(fixture.body, "data-competing", "must not publish");
    assert!(matches!(
        fixture.session.dom().apply(competing, &cancel()),
        Err(DomError::SourceOperationPending)
    ));
    assert_eq!(fixture.session.snapshot().stamp(), pending.stamp());
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(fixture.body)
            .unwrap()
            .attribute(&"data-competing".into()),
        None
    );
    assert!(matches!(
        fixture.resume_without_url(operation, &AtomicBool::new(true)),
        Err(DomSourceRuntimeError::Style(DomStyleRuntimeError::Dom(
            DomError::Cancelled
        )))
    ));
    assert_eq!(fixture.cssom().revision(), original.revision());
    assert_eq!(
        fixture
            .session
            .dom()
            .pending_source_edit()
            .unwrap()
            .operation(),
        operation
    );
    let again = fixture.resume_without_url(operation, &cancel()).unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::HostedChild(again),
        reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
    } = again
    else {
        panic!("same actual URL wait must survive permission drop/cancel: {again:?}")
    };
    assert_eq!(again, request);
    let mut stages = Vec::new();
    let refused = fixture.resume_url(operation, &inputs, &mut csp, &mut stages);
    assert!(
        matches!(
            refused,
            Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                DomHostedError::Limit {
                    resource: DomHostedResource::Registries,
                    limit: 1,
                }
            )))
        ),
        "real later child registry admission required: {refused:?}"
    );
    assert_eq!(stages, [DomSourceUrlStage::IframeDeclaredOrigin]);
    assert_eq!(csp, [operation]);
    assert_eq!(
        check_published_style(&fixture, operation).revision(),
        original.revision()
    );
    fixture
        .session
        .hosted_execution()
        .set_limits(&mut fixture.session.dom(), hosted_limits(32, 32), &cancel())
        .unwrap();
    complete(
        fixture
            .resume_url(operation, &inputs, &mut csp, &mut stages)
            .unwrap(),
        operation,
    );
    assert_eq!(
        stages,
        [
            DomSourceUrlStage::IframeDeclaredOrigin,
            DomSourceUrlStage::FrameIframeSrc
        ]
    );
    assert_eq!(
        csp,
        [operation],
        "acknowledged Style/CSP work must not replay"
    );
    let (id, document) = check_joined(&fixture, &original);
    let history = drain_hosted_fixture_until_idle(&mut fixture);
    assert!(history.iter().any(|step| matches!(step, DomHostedTaskProgress::Executed { document: actual, kind: DomHostedTaskKind::History, .. } if *actual == document)));
    deliver_load(&mut fixture);
    assert_eq!(before.node(fixture.root).unwrap().parent(), None);
    assert_eq!(
        before.node(fixture.root).unwrap().owner_document(),
        Some(fixture.closed)
    );
    assert!(sheet(&before, fixture.style).is_none());
    assert!(before_cssom.sheets().is_empty());
    assert!(matches!(
        pending
            .node(fixture.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet(),
        Err(DomError::IncompleteSourceOperation)
    ));
    let result = fixture.detach().unwrap();
    let DomSourceDriveProgress::Complete(done) = result else {
        panic!("real removal tail: {result:?}")
    };
    assert!(!done.cancellation_requested());
    removal_complete(&fixture, id, &original);
    settle_removal(&mut fixture, id, document);
    teardown(fixture);
}

fn inserted() -> (Fixture, CssomSnapshot, DomHostedNavigableId, DocumentId) {
    let mut fixture = Fixture::new();
    let inputs = UrlInputs::new(&fixture);
    let mut csp = Vec::new();
    let request = fixture.start(&mut csp);
    let original = check_published_style(&fixture, request.operation());
    let mut stages = Vec::new();
    complete(
        fixture
            .resume_url(request.operation(), &inputs, &mut csp, &mut stages)
            .unwrap(),
        request.operation(),
    );
    assert_eq!(csp, [request.operation()]);
    assert_eq!(
        stages,
        [
            DomSourceUrlStage::IframeDeclaredOrigin,
            DomSourceUrlStage::FrameIframeSrc
        ]
    );
    let (id, document) = check_joined(&fixture, &original);
    drain_hosted_fixture_until_idle(&mut fixture);
    deliver_load(&mut fixture);
    (fixture, original, id, document)
}

#[test]
fn mixed_removal_reaches_iframe_destruction_admission_before_the_sibling_stylesheet_removal() {
    let (mut fixture, original, id, document) = inserted();
    let before = fixture.session.snapshot();
    let associated = sheet(&before, fixture.style).unwrap();
    fixture
        .session
        .hosted_execution()
        .set_limits(&mut fixture.session.dom(), hosted_limits(32, 0), &cancel())
        .unwrap();
    let result = fixture.detach();
    assert!(
        matches!(
            result,
            Err(DomSourceRuntimeError::Hosted(
                DomHostedRuntimeError::Admission
            ))
        ),
        "actual child-removal queue must refuse before sibling Style removal: {result:?}"
    );
    let pending = fixture.session.dom().pending_source_edit().unwrap();
    assert!(pending.has_published());
    let removed_prefix = fixture.session.snapshot();
    assert_eq!(removed_prefix.node(fixture.root).unwrap().parent(), None);
    assert_eq!(fixture.cssom().revision(), original.revision());
    assert_eq!(
        fixture.cssom().sheets(),
        std::slice::from_ref(associated.id())
    );
    assert!(
        fixture
            .cssom()
            .sheet(associated.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    // The Source tail has not finished its removal hook. Structural identity
    // and the external CSSOM owner remain readable; a current composed graph
    // read must not certify either active or retiring state at this boundary.
    assert!(matches!(
        removed_prefix.hosted_navigable(id),
        Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
    ));
    assert_eq!(
        removed_prefix
            .node(fixture.iframe)
            .unwrap()
            .owner_document(),
        Some(fixture.top.document())
    );
    assert!(!before.hosted_navigable(id).unwrap().is_retiring());
    assert_eq!(
        before.hosted_navigable(id).unwrap().active_document(),
        Some(document)
    );
    assert!(matches!(
        removed_prefix
            .node(fixture.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet(),
        Err(DomError::IncompleteSourceOperation)
    ));
    assert!(matches!(
        fixture.resume_without_url(pending.operation(), &AtomicBool::new(true)),
        Err(DomSourceRuntimeError::Style(DomStyleRuntimeError::Dom(
            DomError::Cancelled
        )))
    ));
    assert_eq!(fixture.cssom().revision(), original.revision());
    fixture
        .session
        .hosted_execution()
        .set_limits(&mut fixture.session.dom(), hosted_limits(32, 32), &cancel())
        .unwrap();
    complete(
        fixture
            .resume_without_url(pending.operation(), &cancel())
            .unwrap(),
        pending.operation(),
    );
    removal_complete(&fixture, id, &original);
    assert_eq!(sheet(&before, fixture.style), Some(associated));
    assert_eq!(child(&before, fixture.iframe), id);
    assert!(
        before
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    settle_removal(&mut fixture, id, document);
    teardown(fixture);
}

#[test]
fn ordinary_mixed_insert_and_remove_batches_discard_prior_staged_changes_and_publish_neither_owner()
{
    let mut fixture = Fixture::new();
    let before = fixture.session.snapshot();
    let initial = fixture.cssom();
    let initial_usage = fixture.session.dom().usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(fixture.body, "data-staged", "insert must roll back");
    batch.insert_before(fixture.root, fixture.body, None);
    assert!(matches!(
        fixture.session.dom().apply(batch, &cancel()),
        Err(DomError::StyleCompositionRequired(
            HtmlStyleUpdateAction::CheckInlineCsp
        ))
    ));
    let refused = fixture.session.snapshot();
    assert_eq!(refused.stamp(), before.stamp());
    assert_eq!(
        fixture.session.dom().usage().managed_nodes(),
        initial_usage.managed_nodes()
    );
    assert_eq!(
        fixture.session.dom().usage().string_code_units(),
        initial_usage.string_code_units()
    );
    assert_eq!(
        refused
            .node(fixture.body)
            .unwrap()
            .attribute(&"data-staged".into()),
        None
    );
    assert_eq!(refused.node(fixture.root).unwrap().parent(), None);
    assert_eq!(
        refused.node(fixture.root).unwrap().owner_document(),
        Some(fixture.closed)
    );
    assert_eq!(
        refused
            .node(fixture.iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert!(sheet(&refused, fixture.style).is_none());
    assert_eq!(fixture.cssom().revision(), initial.revision());
    assert_eq!(
        fixture.session.style_execution().pending_sheet(0).unwrap(),
        None
    );
    assert!(fixture.session.dom().pending_source_edit().is_none());
    let inputs = UrlInputs::new(&fixture);
    let mut csp = Vec::new();
    let request = fixture.start(&mut csp);
    let original = check_published_style(&fixture, request.operation());
    let mut stages = Vec::new();
    complete(
        fixture
            .resume_url(request.operation(), &inputs, &mut csp, &mut stages)
            .unwrap(),
        request.operation(),
    );
    let (id, document) = check_joined(&fixture, &original);
    drain_hosted_fixture_until_idle(&mut fixture);
    deliver_load(&mut fixture);
    let established = fixture.session.snapshot();
    let associated = sheet(&established, fixture.style).unwrap();
    let established_usage = fixture.session.dom().usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(fixture.body, "data-staged", "remove must roll back");
    batch.detach(fixture.root);
    assert!(matches!(
        fixture.session.dom().apply(batch, &cancel()),
        Err(DomError::HostedSourceUnavailable)
    ));
    let refused = fixture.session.snapshot();
    assert_eq!(refused.stamp(), established.stamp());
    assert_eq!(
        fixture.session.dom().usage().managed_nodes(),
        established_usage.managed_nodes()
    );
    assert_eq!(
        fixture.session.dom().usage().string_code_units(),
        established_usage.string_code_units()
    );
    assert_eq!(
        refused
            .node(fixture.body)
            .unwrap()
            .attribute(&"data-staged".into()),
        None
    );
    assert_eq!(
        refused.node(fixture.root).unwrap().parent(),
        Some(fixture.body)
    );
    assert_eq!(sheet(&refused, fixture.style), Some(associated));
    assert_eq!(fixture.cssom().revision(), original.revision());
    assert_eq!(child(&refused, fixture.iframe), id);
    assert!(!refused.hosted_navigable(id).unwrap().is_retiring());
    assert!(fixture.session.dom().pending_source_edit().is_none());
    let result = fixture.detach().unwrap();
    let DomSourceDriveProgress::Complete(_) = result else {
        panic!("real composed removal: {result:?}")
    };
    removal_complete(&fixture, id, &original);
    settle_removal(&mut fixture, id, document);
    teardown(fixture);
}
