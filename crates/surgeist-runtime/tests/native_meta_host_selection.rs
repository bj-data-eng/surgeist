//! Runtime selections coexist with native Meta's sealed no-UI consumer domain.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlReferrerPolicy;
use surgeist_runtime::*;

fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: Default::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 16,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}

#[test]
fn native_meta_history_and_empty_theme_scans_preserve_runtime_selection_associations() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(64, 8192, 128, 64, 32, 8, 262144)
            .unwrap()
            .with_meta_processing_work(1_000_000),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let html = batch.create_element_ns(&document, Namespace::html(), "html");
    let head = batch.create_element_ns(&document, Namespace::html(), "head");
    let body = batch.create_element_ns(&document, Namespace::html(), "body");
    let a = batch.create_element_ns(&document, Namespace::html(), "div");
    let b = batch.create_element_ns(&document, Namespace::html(), "span");
    let language = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.set_attribute(&language, "http-equiv", "content-language");
    batch.set_attribute(&language, "content", "fr");
    let referrer = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.set_attribute(&referrer, "name", "referrer");
    batch.set_attribute(&referrer, "content", "no-referrer");
    batch.insert_before(&a, &body, None);
    batch.insert_before(&b, &body, None);
    batch.insert_before(&head, &html, None);
    batch.insert_before(&body, &html, None);
    batch.insert_before(&html, &document, None);
    let setup = dom.apply(batch, &cancel).unwrap();
    let document = setup.document(&document).unwrap();
    let head = setup.created(&head).unwrap().unwrap();
    let a = setup.created(&a).unwrap().unwrap();
    let b = setup.created(&b).unwrap().unwrap();
    let language = setup.created(&language).unwrap().unwrap();
    let referrer = setup.created(&referrer).unwrap().unwrap();
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let old = session.snapshot();
    assert!(!old.document(document).unwrap().is_fully_active().unwrap());
    assert!(
        old.document(document)
            .unwrap()
            .metadata()
            .meta_state()
            .unwrap()
            .pragma_language()
            .is_none()
    );
    assert_eq!(
        old.document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .referrer_policy(),
        Some(HtmlReferrerPolicy::StrictOriginWhenCrossOrigin)
    );

    let mut root = SurfaceRoot::new(RootId::new("native-meta-selections"));
    for id in [ElementId::from_u64(1), ElementId::from_u64(2)] {
        root.register_element(ElementRegistration::try_new(id, [ElementPhase::Target]).unwrap())
            .unwrap();
    }
    let mut runtime: Runtime = Runtime::new((), ());
    let surface = runtime
        .register_surface(
            UiSurface::try_new(SurfaceId::from_u64(7), WindowId::from_u64(9), root).unwrap(),
        )
        .unwrap();
    let a_ref = SurfaceElementRef::new(surface, ElementId::from_u64(1));
    let b_ref = SurfaceElementRef::new(surface, ElementId::from_u64(2));
    let associations = vec![(a_ref, a), (b_ref, b)];
    let limits = SurfaceDomBindingLimits::new(8, 256);
    let mut host = runtime
        .bind_dom_host(
            surface,
            old.document(document).unwrap(),
            associations.clone(),
            DomHostDomains::FocusAndHover,
            limits,
        )
        .unwrap();
    runtime.set_focus(surface, Some(a_ref)).unwrap();
    runtime.set_hover(surface, Some(b_ref)).unwrap();
    let selected = runtime
        .capture_dom_host(&mut host, old.document(document).unwrap())
        .unwrap();
    let expected = DomHostObservations::FocusAndHover {
        focus: Observed::Supported(Some(a)),
        hover: Observed::Supported(Some(b)),
    };
    assert_eq!(selected.publication().capture().observations(), &expected);

    // These ordinary selection associations do not install a Meta UI consumer
    // or grant HTML content system focus. Actual native insertion still scans
    // the complete tree, whose theme-color/color-scheme candidate sets are empty.
    let mut batch = DomBatch::new();
    batch.insert_before(language, head, None);
    batch.insert_before(referrer, head, None);
    session.dom().apply(batch, &cancel).unwrap();
    let current = session.snapshot();
    let metadata = current.document(document).unwrap().metadata();
    assert_eq!(
        metadata.meta_state().unwrap().pragma_language(),
        Some(&"fr".into())
    );
    assert_eq!(
        metadata.policy_container().referrer_policy(),
        Some(HtmlReferrerPolicy::NoReferrer)
    );
    assert_eq!(
        current.node(head).unwrap().children(),
        &[language, referrer]
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert!(
        !current
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        runtime
            .surface(surface.surface_id())
            .unwrap()
            .focused_element(),
        Some(a_ref)
    );
    assert_eq!(
        runtime
            .surface(surface.surface_id())
            .unwrap()
            .hovered_element(),
        Some(b_ref)
    );
    // Requalify only the changed canonical basis with the same real associations.
    runtime
        .rebind_dom_host(
            &mut host,
            current.document(document).unwrap(),
            associations,
            limits,
        )
        .unwrap();
    let after = runtime
        .capture_dom_host(&mut host, current.document(document).unwrap())
        .unwrap();
    assert_eq!(after.publication().capture().observations(), &expected);
    runtime.set_focus(surface, Some(b_ref)).unwrap();
    runtime.set_hover(surface, Some(a_ref)).unwrap();
    let swapped = runtime
        .capture_dom_host(&mut host, current.document(document).unwrap())
        .unwrap();
    assert_eq!(
        swapped.publication().capture().observations(),
        &DomHostObservations::FocusAndHover {
            focus: Observed::Supported(Some(b)),
            hover: Observed::Supported(Some(a)),
        }
    );
    assert_eq!(selected.publication().capture().observations(), &expected);
    assert!(
        old.document(document)
            .unwrap()
            .metadata()
            .meta_state()
            .unwrap()
            .pragma_language()
            .is_none()
    );
    assert_eq!(
        old.document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .referrer_policy(),
        Some(HtmlReferrerPolicy::StrictOriginWhenCrossOrigin)
    );
    assert!(old.node(head).unwrap().children().is_empty());
    assert_eq!(session.snapshot().stamp(), current.stamp());
    runtime.remove_surface(surface).unwrap();
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
