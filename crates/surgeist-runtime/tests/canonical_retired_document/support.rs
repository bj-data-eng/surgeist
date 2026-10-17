use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_runtime::*;

pub(super) fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

pub(super) fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4)
}

fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}

pub(super) fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    snapshot
        .node(html)
        .unwrap()
        .children()
        .iter()
        .copied()
        .find(|id| {
            snapshot
                .node(*id)
                .unwrap()
                .as_element()
                .is_some_and(|element| {
                    element.name().namespace() == &Namespace::html()
                        && element.name().local_name() == &DocumentString::from("body")
                })
        })
        .expect("the actual HTML creator installs a body")
}

pub(super) struct HostedFixture {
    pub session: DomSession,
    pub top: DomHostedTopLevel,
    pub body: NodeId,
    pub closed: DocumentId,
    pub original: DomSnapshot,
    pub registry: DomRegistryBinding,
    pub retirement: Option<DomHostedDestructionQueued>,
}

impl HostedFixture {
    pub fn active() -> Self {
        Self::active_with_platform(DomNativeInputPlatform::headless())
    }

    pub fn active_with_platform(platform: DomNativeInputPlatform) -> Self {
        let dom = Dom::with_native_input_platform(
            DomLimits::new(512, 200_000, 1024, 1024, 1024, 64, 4_000_000)
                .unwrap()
                .with_dispatch_custody(
                    DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000)
                        .unwrap(),
                )
                .with_details_toggle_intentions(32)
                .unwrap(),
            platform,
        )
        .unwrap();
        let mut session = DomSession::new(dom, limits()).unwrap();
        let mut hosted = session.hosted_execution();
        let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
        let top = hosted
            .create_top_level(&mut session.dom(), host, &cancel())
            .unwrap();
        let mut batch = DomBatch::new();
        let closed = batch.create_html_document(top.document(), None);
        let commit = session.dom().apply(batch, &cancel()).unwrap();
        let closed = commit.document(&closed).unwrap();
        let original = session.snapshot();
        let body = body(&original, top.document());
        let registry = original
            .document(top.document())
            .unwrap()
            .metadata()
            .custom_element_registry();
        assert!(matches!(registry, DomRegistryBinding::Registry(_)));
        assert!(
            original
                .document(top.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        assert!(
            !original
                .document(closed)
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        Self {
            session,
            top,
            body,
            closed,
            original,
            registry,
            retirement: None,
        }
    }

    pub fn retired() -> Self {
        let mut fixture = Self::active();
        fixture.retire();
        fixture
    }

    pub fn retire(&mut self) {
        assert!(self.retirement.is_none());
        let mut hosted = self.session.hosted_execution();
        let queued = hosted
            .destroy_top_level(&mut self.session.dom(), self.top.navigable(), &cancel())
            .unwrap();
        let mut last = None;
        for _ in 0..32 {
            let progress = hosted
                .run_next(&mut self.session.dom(), queued.traversable(), &cancel())
                .unwrap();
            last = Some(progress);
            match progress {
                DomHostedTaskProgress::Complete { operation }
                    if operation == queued.operation() =>
                {
                    self.retirement = Some(queued);
                    self.assert_retired();
                    return;
                }
                // The traversable also pumps its separately issued cleanup
                // operations. Their IDs belong to those actual tasks, not the
                // outer invocation. Only its exact Complete ends this pump.
                DomHostedTaskProgress::Complete { .. }
                | DomHostedTaskProgress::Executed { .. }
                | DomHostedTaskProgress::CanceledDocumentDestroyed { .. }
                | DomHostedTaskProgress::Pending { .. } => {}
                other => panic!("actual empty-top teardown must make progress: {other:?}"),
            }
        }
        panic!("the actual empty-top destruction did not complete within 32 queue steps: {last:?}");
    }

    pub fn assert_retired(&mut self) {
        let current = self.session.snapshot();
        assert!(
            !current
                .document(self.top.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        assert_eq!(
            current.global_document(self.top.global()).unwrap(),
            self.top.document()
        );
        assert_eq!(
            current.relevant_global(self.body).unwrap(),
            Some(self.top.global())
        );
        assert_eq!(
            current
                .document(self.top.document())
                .unwrap()
                .metadata()
                .custom_element_registry(),
            self.registry
        );
        assert!(
            self.original
                .document(self.top.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        let hosted = self.session.hosted_execution();
        assert!(
            hosted
                .settings(&self.session.dom(), self.top.global(), &cancel())
                .is_err()
        );
    }
}
