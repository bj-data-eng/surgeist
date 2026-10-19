use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomError, DomHostChange, DomHostChangeCoverage, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostRecomputeReason, DomLimits, NodeId,
    Observed,
};
use surgeist_runtime::{
    ElementId, ElementPhase, ElementRegistration, RootId, Runtime, RuntimeDomHostError,
    SurfaceDomBindingLimits, SurfaceElementRef, SurfaceErrorCode, SurfaceId, SurfaceLifecycle,
    SurfaceRef, SurfaceRoot, UiSurface, WindowId,
};

fn document() -> (Dom, DocumentId, [NodeId; 2]) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            128,
            0,
            0,
            16,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let a = batch.create_element(&document, "A");
    let b = batch.create_element(&document, "B");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        [
            commit.created(&a).unwrap().unwrap(),
            commit.created(&b).unwrap().unwrap(),
        ],
    )
}
fn runtime(ids: &[u64]) -> (Runtime, SurfaceRef, Vec<SurfaceElementRef>) {
    let mut root = SurfaceRoot::new(RootId::new("root"));
    for &id in ids {
        root.register_element(
            ElementRegistration::try_new(ElementId::from_u64(id), [ElementPhase::Target]).unwrap(),
        )
        .unwrap();
    }
    let mut runtime = Runtime::new((), ());
    let reference = runtime
        .register_surface(
            UiSurface::try_new(SurfaceId::from_u64(37), WindowId::from_u64(91), root).unwrap(),
        )
        .unwrap();
    let elements = ids
        .iter()
        .map(|&id| SurfaceElementRef::new(reference, ElementId::from_u64(id)))
        .collect();
    (runtime, reference, elements)
}
fn limits() -> SurfaceDomBindingLimits {
    SurfaceDomBindingLimits::new(32, 1024)
}

fn registered_root(name: &str, element: ElementId) -> SurfaceRoot {
    let mut root = SurfaceRoot::new(RootId::new(name));
    root.register_element(ElementRegistration::try_new(element, [ElementPhase::Target]).unwrap())
        .unwrap();
    root
}

#[test]
fn rejected_fresh_surface_preserves_host_associations_captures_and_predecessor_coverage() {
    let (dom, id, [a, _]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (mut runtime, surface, elements) = runtime(&[1]);
    let mut host = runtime
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], a)],
            DomHostDomains::FocusAndHover,
            limits(),
        )
        .unwrap();
    let initial = runtime.capture_dom_host(&mut host, document).unwrap();
    runtime.set_focus(surface, Some(elements[0])).unwrap();
    let selected = runtime.capture_dom_host(&mut host, document).unwrap();
    let error = runtime
        .update_surface(surface, |staged| {
            *staged = UiSurface::try_new(
                surface.surface_id(),
                WindowId::from_u64(91),
                registered_root("unpublished", ElementId::from_u64(2)),
            )?;
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.code(), SurfaceErrorCode::InvalidSurfaceReplacement);
    assert_eq!(runtime.surface_ref(surface.surface_id()), Some(surface));
    let retained = runtime.surface(surface.surface_id()).unwrap();
    assert_eq!(retained.root().id(), &RootId::new("root"));
    assert!(
        retained
            .root()
            .elements()
            .get(ElementId::from_u64(1))
            .is_some()
    );
    assert!(
        retained
            .root()
            .elements()
            .get(ElementId::from_u64(2))
            .is_none()
    );
    let after_rejection = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        after_rejection.publication().capture(),
        selected.publication().capture()
    );
    assert_eq!(
        host.changes_since(Some(initial.publication().capture().stamp()))
            .unwrap(),
        *selected.publication().changes()
    );
    assert!(
        matches!(after_rejection.publication().changes(), DomHostChangeCoverage::Complete(changes) if changes.changes().next().is_none())
    );

    // A legitimate no-op draft is accepted without changing host facts/history.
    runtime.update_surface(surface, |_| Ok(())).unwrap();
    assert_eq!(
        runtime
            .capture_dom_host(&mut host, document)
            .unwrap()
            .publication()
            .capture(),
        selected.publication().capture()
    );
    assert_eq!(
        host.changes_since(Some(initial.publication().capture().stamp()))
            .unwrap(),
        *selected.publication().changes()
    );
    // The public producer mutation alone advances the host clock under the
    // retained source identity and canonical basis.
    runtime.set_hover(surface, Some(elements[0])).unwrap();
    let hovered = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        hovered.publication().capture().observations(),
        &facts(Some(a), Some(a))
    );
    assert_eq!(
        hovered
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        2
    );
    assert_eq!(
        hovered.publication().capture().stamp().source(),
        selected.publication().capture().stamp().source()
    );
    assert_eq!(
        selected.publication().capture().observations(),
        &facts(Some(a), None)
    );
}

#[test]
fn rejected_escaped_stage_preserves_current_binding_and_cannot_revive_old_host_source() {
    let (dom, id, [a, b]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (mut runtime, surface, elements) = runtime(&[1]);
    runtime.set_focus(surface, Some(elements[0])).unwrap();
    let mut old_host = runtime
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let old_capture = runtime.capture_dom_host(&mut old_host, document).unwrap();
    let mut escaped = None;
    let error = runtime
        .update_surface(surface, |staged| {
            let replacement = UiSurface::try_new(
                surface.surface_id(),
                WindowId::from_u64(91),
                registered_root("unpublished", ElementId::from_u64(2)),
            )?;
            escaped = Some(std::mem::replace(staged, replacement));
            staged.transition_to(SurfaceLifecycle::Created).map(|_| ())
        })
        .unwrap_err();
    assert_eq!(error.code(), SurfaceErrorCode::InvalidLifecycleTransition);
    runtime
        .update_surface(surface, |staged| {
            staged
                .replace_root(registered_root("current", ElementId::from_u64(2)))
                .map(|_| ())
        })
        .unwrap();
    let current = runtime.surface_ref(surface.surface_id()).unwrap();
    let target = SurfaceElementRef::new(current, ElementId::from_u64(2));
    let mut current_host = runtime
        .bind_dom_host(
            current,
            document,
            vec![(target, b)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let initial = runtime
        .capture_dom_host(&mut current_host, document)
        .unwrap();
    runtime.set_focus(current, Some(target)).unwrap();
    let selected = runtime
        .capture_dom_host(&mut current_host, document)
        .unwrap();
    let error = runtime
        .update_surface(current, |staged| {
            *staged = escaped.take().unwrap();
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.code(), SurfaceErrorCode::InvalidSurfaceReplacement);
    assert_eq!(runtime.surface_ref(surface.surface_id()), Some(current));
    let retained = runtime.surface(surface.surface_id()).unwrap();
    assert_eq!(retained.root().id(), &RootId::new("current"));
    assert!(
        retained
            .root()
            .elements()
            .get(ElementId::from_u64(1))
            .is_none()
    );
    assert!(
        retained
            .root()
            .elements()
            .get(ElementId::from_u64(2))
            .is_some()
    );
    assert_eq!(
        runtime
            .capture_dom_host(&mut current_host, document)
            .unwrap()
            .publication()
            .capture(),
        selected.publication().capture()
    );
    assert_eq!(
        current_host
            .changes_since(Some(initial.publication().capture().stamp()))
            .unwrap(),
        *selected.publication().changes()
    );
    assert_eq!(
        selected.publication().capture().observations(),
        &DomHostObservations::Focus(Observed::Supported(Some(b)))
    );
    assert_eq!(
        selected
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        1
    );
    assert!(
        matches!(runtime.capture_dom_host(&mut old_host, document), Err(RuntimeDomHostError::Surface(error)) if error.code() == SurfaceErrorCode::StaleSurfaceGeneration)
    );
    assert_ne!(
        selected.publication().capture().stamp().source(),
        old_capture.publication().capture().stamp().source()
    );
    assert_eq!(old_host.latest(), Some(old_capture.publication().capture()));
    assert_eq!(
        old_capture.publication().capture().observations(),
        &DomHostObservations::Focus(Observed::Supported(Some(a)))
    );
}
fn facts(focus: Option<NodeId>, hover: Option<NodeId>) -> DomHostObservations {
    DomHostObservations::FocusAndHover {
        focus: Observed::Supported(focus),
        hover: Observed::Supported(hover),
    }
}

#[test]
fn current_created_empty_surface_supplies_supported_absence_in_all_fixed_configurations() {
    let (dom, id, _) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (runtime, surface, _) = runtime(&[]);
    for (domains, expected) in [
        (
            DomHostDomains::Focus,
            DomHostObservations::Focus(Observed::Supported(None)),
        ),
        (
            DomHostDomains::Hover,
            DomHostObservations::Hover(Observed::Supported(None)),
        ),
        (DomHostDomains::FocusAndHover, facts(None, None)),
    ] {
        let mut host = runtime
            .bind_dom_host(
                surface,
                document,
                vec![],
                domains,
                SurfaceDomBindingLimits::new(0, 0),
            )
            .unwrap();
        let first = runtime.capture_dom_host(&mut host, document).unwrap();
        assert_eq!(first.surface(), surface);
        assert_eq!(first.window(), WindowId::from_u64(91));
        assert_eq!(
            first.publication().capture().stamp().document(),
            document.stamp()
        );
        assert_eq!(first.publication().capture().observations(), &expected);
        assert_eq!(
            first.publication().capture().stamp().host_revision().get(),
            0
        );
        let second = runtime.capture_dom_host(&mut host, document).unwrap();
        assert_eq!(
            first.publication().capture(),
            second.publication().capture()
        );
        assert!(
            matches!(second.publication().changes(), DomHostChangeCoverage::Complete(changes) if changes.changes().next().is_none())
        );
    }
}

#[test]
fn actual_focus_hover_join_uses_explicit_unrelated_ids_and_preserves_host_history_across_rendering()
{
    let (dom, id, [a, b]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (mut runtime, surface, elements) = runtime(&[9001, 4]);
    let mut host = runtime
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], b), (elements[1], a)],
            DomHostDomains::FocusAndHover,
            limits(),
        )
        .unwrap();
    let first = runtime.capture_dom_host(&mut host, document).unwrap();
    runtime.set_focus(surface, Some(elements[0])).unwrap();
    runtime.set_hover(surface, Some(elements[1])).unwrap();
    let selected = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        selected.publication().capture().observations(),
        &facts(Some(b), Some(a))
    );
    assert_eq!(
        selected.publication().capture().stamp().document(),
        document.stamp()
    );
    assert_eq!(
        selected
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        1
    );
    let DomHostChangeCoverage::Complete(changes) = selected.publication().changes() else {
        panic!("expected complete selection changes")
    };
    assert_eq!(
        changes.changes().copied().collect::<Vec<_>>(),
        [
            DomHostChange::Target {
                domain: DomHostDomain::ElementFocusSelection,
                previous: None,
                current: Some(b)
            },
            DomHostChange::Target {
                domain: DomHostDomain::ElementHoverSelection,
                previous: None,
                current: Some(a)
            },
        ]
    );
    assert!(
        !runtime
            .set_focus(surface, Some(elements[0]))
            .unwrap()
            .changed()
    );
    runtime
        .update_surface(surface, |current| current.ready().map(|_| ()))
        .unwrap();
    let frame = runtime.begin_render(surface).unwrap().into_frame();
    let acknowledgement = runtime.mark_rendered(frame).unwrap();
    assert!(acknowledgement.consumed_invalidations() > 0);
    let after_render = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        after_render.publication().capture(),
        selected.publication().capture()
    );
    assert_eq!(
        host.changes_since(Some(first.publication().capture().stamp()))
            .unwrap(),
        *selected.publication().changes()
    );
    runtime.set_focus(surface, None).unwrap();
    runtime.set_hover(surface, None).unwrap();
    let cleared = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        cleared.publication().capture().observations(),
        &facts(None, None)
    );
    assert_eq!(
        cleared
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        2
    );
    assert_eq!(
        selected.publication().capture().observations(),
        &facts(Some(b), Some(a))
    );
}

#[test]
fn coinciding_numeric_surface_ids_do_not_authorize_another_runtime_to_use_the_adapter() {
    let (dom, id, [a, _]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (first, surface, elements) = runtime(&[12]);
    let (second, other_surface, other_elements) = runtime(&[12]);
    assert_eq!(surface, other_surface);
    assert_eq!(elements, other_elements);
    let mut host = first
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let old = first.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        second.capture_dom_host(&mut host, document),
        Err(RuntimeDomHostError::ForeignRuntime)
    );
    assert_eq!(
        second.rebind_dom_host(&mut host, document, vec![(elements[0], a)], limits()),
        Err(RuntimeDomHostError::ForeignRuntime)
    );
    assert_eq!(host.latest(), Some(old.publication().capture()));
}

#[test]
fn associations_require_actual_registered_unique_same_document_elements() {
    let (mut dom, id, [a, b]) = document();
    let mut batch = DomBatch::new();
    let text = batch.create_text(id, "not an element");
    let other = batch.create_document();
    let wrong_document = batch.create_element(&other, "Other");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let wrong_document = commit.created(&wrong_document).unwrap().unwrap();
    let (foreign, _, [foreign_node, _]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (runtime, surface, elements) = runtime(&[5, 8]);
    let missing = runtime
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap_err();
    assert_eq!(
        missing,
        RuntimeDomHostError::MissingAssociation(elements[1])
    );
    assert_eq!(
        runtime
            .bind_dom_host(
                surface,
                document,
                vec![(elements[0], a), (elements[0], b)],
                DomHostDomains::Focus,
                limits()
            )
            .unwrap_err(),
        RuntimeDomHostError::DuplicateSurfaceElement(elements[0])
    );
    assert_eq!(
        runtime
            .bind_dom_host(
                surface,
                document,
                vec![(elements[0], a), (elements[1], a)],
                DomHostDomains::Focus,
                limits()
            )
            .unwrap_err(),
        RuntimeDomHostError::DuplicateDomElement(a)
    );
    for (node, error) in [
        (text, DomError::WrongNodeKind),
        (wrong_document, DomError::WrongDocument),
        (foreign_node, DomError::ForeignOwner),
    ] {
        assert_eq!(
            runtime
                .bind_dom_host(
                    surface,
                    document,
                    vec![(elements[0], node), (elements[1], b)],
                    DomHostDomains::Focus,
                    limits()
                )
                .unwrap_err(),
            RuntimeDomHostError::DomSubject { node, error }
        );
    }
    let unknown = SurfaceElementRef::new(surface, ElementId::from_u64(600));
    assert!(
        matches!(runtime.bind_dom_host(surface, document, vec![(unknown, a)], DomHostDomains::Focus, limits()), Err(RuntimeDomHostError::Surface(error)) if error.code() == SurfaceErrorCode::UnknownElement)
    );
    drop(foreign);
}

#[test]
fn finite_association_admission_has_independent_exact_and_one_short_boundaries() {
    let (dom, id, [a, _]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (runtime, surface, elements) = runtime(&[17]);
    // One pair dispatch, registration validation, DOM validation, registry
    // dispatch and registry association comparison require exactly five credits.
    let associations = vec![(elements[0], a)];
    assert_eq!(associations.capacity(), 1);
    let mut host = runtime
        .bind_dom_host(
            surface,
            document,
            associations,
            DomHostDomains::Focus,
            SurfaceDomBindingLimits::new(1, 5),
        )
        .unwrap();
    let before = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        runtime.rebind_dom_host(
            &mut host,
            document,
            vec![(elements[0], a)],
            SurfaceDomBindingLimits::new(0, 5)
        ),
        Err(RuntimeDomHostError::AssociationCapacity {
            limit: 0,
            required: 1
        })
    );
    assert_eq!(
        runtime.rebind_dom_host(
            &mut host,
            document,
            vec![(elements[0], a)],
            SurfaceDomBindingLimits::new(1, 4)
        ),
        Err(RuntimeDomHostError::ValidationWork {
            limit: 4,
            required: 5
        })
    );
    assert_eq!(host.latest(), Some(before.publication().capture()));
    assert_eq!(host.document(), document.stamp());
    assert_eq!(
        runtime
            .capture_dom_host(&mut host, document)
            .unwrap()
            .publication()
            .capture(),
        before.publication().capture()
    );
}

#[test]
fn canonical_changes_require_rebind_and_keep_equal_host_revision_with_explicit_fallback() {
    let (mut dom, id, [a, b]) = document();
    let old = dom.snapshot();
    let old_document = old.document(id).unwrap();
    let (mut runtime, surface, elements) = runtime(&[11]);
    runtime.set_focus(surface, Some(elements[0])).unwrap();
    let mut host = runtime
        .bind_dom_host(
            surface,
            old_document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let before = runtime.capture_dom_host(&mut host, old_document).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "key", "value");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let new = dom.snapshot();
    let document = new.document(id).unwrap();
    assert_eq!(
        runtime.capture_dom_host(&mut host, document),
        Err(RuntimeDomHostError::BindingDomBasisMismatch {
            expected: old_document.stamp(),
            actual: document.stamp()
        })
    );
    assert_eq!(host.latest(), Some(before.publication().capture()));
    runtime
        .rebind_dom_host(&mut host, document, vec![(elements[0], a)], limits())
        .unwrap();
    let recaptured = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        recaptured
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        0
    );
    assert_eq!(
        recaptured.publication().capture().stamp().source(),
        before.publication().capture().stamp().source()
    );
    assert!(matches!(
        recaptured.publication().changes(),
        DomHostChangeCoverage::FullRecomputeRequired {
            reason: DomHostRecomputeReason::DomBasisChanged,
            ..
        }
    ));
    assert_eq!(
        runtime.rebind_dom_host(&mut host, old_document, vec![(elements[0], a)], limits()),
        Err(RuntimeDomHostError::Host(DomHostError::StaleDomRevision {
            current: new.revision(),
            actual: old.revision()
        }))
    );
    // Changing the explicit association is observable even if the surface selection
    // itself stays fixed. It remains the same coherent publisher/source.
    runtime
        .rebind_dom_host(&mut host, document, vec![(elements[0], b)], limits())
        .unwrap();
    let rebound = runtime.capture_dom_host(&mut host, document).unwrap();
    assert_eq!(
        rebound.publication().capture().observations(),
        &DomHostObservations::Focus(Observed::Supported(Some(b)))
    );
    assert_eq!(
        rebound
            .publication()
            .capture()
            .stamp()
            .host_revision()
            .get(),
        1
    );
}

#[test]
fn root_replacement_terminal_lifecycle_and_reregistration_reject_old_bindings() {
    let (dom, id, [a, _]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (mut runtime, surface, elements) = runtime(&[6]);
    let mut host = runtime
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let before = runtime.capture_dom_host(&mut host, document).unwrap();
    runtime
        .update_surface(surface, |current| {
            current
                .replace_root(SurfaceRoot::new(RootId::new("replacement")))
                .map(|_| ())
        })
        .unwrap();
    assert!(
        matches!(runtime.capture_dom_host(&mut host, document), Err(RuntimeDomHostError::Surface(error)) if error.code() == SurfaceErrorCode::StaleSurfaceGeneration)
    );
    assert_eq!(host.latest(), Some(before.publication().capture()));
    let current = runtime.surface_ref(surface.surface_id()).unwrap();
    let mut empty = runtime
        .bind_dom_host(current, document, vec![], DomHostDomains::Focus, limits())
        .unwrap();
    runtime
        .update_surface(current, |surface| surface.closed().map(|_| ()))
        .unwrap();
    assert!(
        matches!(runtime.capture_dom_host(&mut empty, document), Err(RuntimeDomHostError::Surface(error)) if error.code() == SurfaceErrorCode::TerminalSurface)
    );
    runtime.remove_surface(current).unwrap();
    assert!(
        matches!(runtime.capture_dom_host(&mut empty, document), Err(RuntimeDomHostError::Surface(error)) if error.code() == SurfaceErrorCode::UnknownSurface)
    );
    runtime
        .register_surface(
            UiSurface::try_new(
                surface.surface_id(),
                WindowId::from_u64(91),
                SurfaceRoot::new(RootId::new("registered again")),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(
        matches!(runtime.capture_dom_host(&mut empty, document), Err(RuntimeDomHostError::Surface(error)) if error.code() == SurfaceErrorCode::StaleSurfaceGeneration)
    );
}

#[test]
fn destroying_a_bound_node_rejects_rebind_atomically_and_old_capture_stays_exact() {
    let (mut dom, id, [a, b]) = document();
    let old = dom.snapshot();
    let old_document = old.document(id).unwrap();
    let (mut runtime, surface, elements) = runtime(&[31]);
    runtime.set_focus(surface, Some(elements[0])).unwrap();
    let mut host = runtime
        .bind_dom_host(
            surface,
            old_document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let captured = runtime.capture_dom_host(&mut host, old_document).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(a);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let new = dom.snapshot();
    let document = new.document(id).unwrap();
    assert_eq!(
        runtime.rebind_dom_host(&mut host, document, vec![(elements[0], a)], limits()),
        Err(RuntimeDomHostError::DomSubject {
            node: a,
            error: DomError::StaleNode
        })
    );
    assert_eq!(host.document(), old_document.stamp());
    assert_eq!(host.latest(), Some(captured.publication().capture()));
    runtime
        .rebind_dom_host(&mut host, document, vec![(elements[0], b)], limits())
        .unwrap();
    assert_eq!(
        runtime
            .capture_dom_host(&mut host, document)
            .unwrap()
            .publication()
            .capture()
            .observations(),
        &DomHostObservations::Focus(Observed::Supported(Some(b)))
    );
    assert_eq!(
        captured.publication().capture().validate_for(old_document),
        Ok(())
    );
    assert_eq!(
        captured.publication().capture().observations(),
        &DomHostObservations::Focus(Observed::Supported(Some(a)))
    );
}
