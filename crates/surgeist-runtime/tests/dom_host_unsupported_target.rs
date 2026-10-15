#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;
use surgeist_dom::{Dom, DomBatch, DomHostDomains, DomHostObservations, DomLimits, Observed};
use surgeist_runtime::{
    ElementId, ElementPhase, ElementRegistration, RootId, Runtime, RuntimeDomHostError,
    SurfaceDomBindingLimits, SurfaceElementRef, SurfaceId, SurfaceRoot, UiSurface, WindowId,
};

#[test]
fn target_configuration_is_rejected_before_association_admission_without_changing_existing_capture()
{
    let mut dom = Dom::new(DomLimits::new(16, 4096, 32, 0, 0, 4, 8192).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element(&document, "target");
    batch.set_attribute(&element, "id", "target");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = commit.created(&element).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let document = snapshot
        .document(commit.document(&document).unwrap())
        .unwrap();
    let mut root = SurfaceRoot::new(RootId::new("root"));
    let element = ElementId::from_u64(7);
    root.register_element(ElementRegistration::try_new(element, [ElementPhase::Target]).unwrap())
        .unwrap();
    let mut runtime: Runtime = Runtime::new((), ());
    let surface = runtime
        .register_surface(
            UiSurface::try_new(SurfaceId::from_u64(11), WindowId::from_u64(13), root).unwrap(),
        )
        .unwrap();
    let reference = SurfaceElementRef::new(surface, element);
    let mut host = runtime
        .bind_dom_host(
            surface,
            document,
            vec![(reference, node)],
            DomHostDomains::FocusAndHover,
            SurfaceDomBindingLimits::new(8, 128),
        )
        .unwrap();
    let initial = runtime.capture_dom_host(&mut host, document).unwrap();
    runtime.set_focus(surface, Some(reference)).unwrap();
    runtime.set_hover(surface, Some(reference)).unwrap();
    let selected = runtime.capture_dom_host(&mut host, document).unwrap();
    let invalidations = runtime
        .surface(surface.surface_id())
        .unwrap()
        .invalidations()
        .to_vec();

    // Even zero association/work admission and a missing registry association
    // cannot turn an unsupported producer into a target-none publication.
    for associations in [vec![], vec![(reference, node)]] {
        let error = runtime
            .bind_dom_host(
                surface,
                document,
                associations,
                DomHostDomains::DocumentTarget,
                SurfaceDomBindingLimits::new(0, 0),
            )
            .unwrap_err();
        assert_eq!(
            error,
            RuntimeDomHostError::UnsupportedDomains(DomHostDomains::DocumentTarget)
        );
        assert_eq!(host.latest(), Some(selected.publication().capture()));
        assert_eq!(
            host.changes_since(Some(initial.publication().capture().stamp()))
                .unwrap(),
            *selected.publication().changes()
        );
    }
    let retained = runtime.surface(surface.surface_id()).unwrap();
    assert_eq!(retained.focused_element(), Some(reference));
    assert_eq!(retained.hovered_element(), Some(reference));
    assert_eq!(retained.invalidations(), invalidations);
    assert_eq!(runtime.surface_ref(surface.surface_id()), Some(surface));
    let current = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        current.publication().capture(),
        selected.publication().capture()
    );
    assert_eq!(
        current.publication().capture().observations(),
        &DomHostObservations::FocusAndHover {
            focus: Observed::Supported(Some(node)),
            hover: Observed::Supported(Some(node)),
        }
    );
    assert_eq!(
        current
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        1
    );
    assert_eq!(
        host.changes_since(Some(initial.publication().capture().stamp()))
            .unwrap(),
        *selected.publication().changes()
    );
    assert_eq!(snapshot.stamp(), dom.snapshot().stamp());
}
