#![forbid(unsafe_code)]

use surgeist_runtime::{
    ElementId, ElementPhase, ElementRegistration, RootId, Runtime, SurfaceElementRef,
    SurfaceErrorCode, SurfaceId, SurfaceLifecycle, SurfaceRoot, UiSurface, WindowId,
};

fn root(name: &str, element: ElementId) -> SurfaceRoot {
    let mut root = SurfaceRoot::new(RootId::new(name));
    root.register_element(ElementRegistration::try_new(element, [ElementPhase::Target]).unwrap())
        .unwrap();
    root
}

#[test]
fn escaped_rejected_stage_cannot_resurrect_a_retired_root_generation() {
    let id = SurfaceId::from_u64(37);
    let window = WindowId::from_u64(91);
    let old_element = ElementId::from_u64(1);
    let new_element = ElementId::from_u64(2);
    let mut runtime = Runtime::<()>::new((), ());
    let original = runtime
        .register_surface(UiSurface::try_new(id, window, root("original", old_element)).unwrap())
        .unwrap();
    let old_target = SurfaceElementRef::new(original, old_element);
    runtime.set_focus(original, Some(old_target)).unwrap();
    runtime.set_hover(original, Some(old_target)).unwrap();

    let mut escaped = None;
    let rejected = runtime
        .update_surface(original, |staged| {
            let replacement = UiSurface::try_new(id, window, root("unpublished", new_element))?;
            escaped = Some(std::mem::replace(staged, replacement));
            // Created -> Created is an actual owner-rejected lifecycle transition.
            staged.transition_to(SurfaceLifecycle::Created).map(|_| ())
        })
        .unwrap_err();
    assert_eq!(
        rejected.code(),
        SurfaceErrorCode::InvalidLifecycleTransition
    );
    assert_eq!(runtime.surface_ref(id), Some(original));
    assert_eq!(
        runtime.surface(id).unwrap().root().id(),
        &RootId::new("original")
    );
    assert_eq!(
        runtime.surface(id).unwrap().focused_element(),
        Some(old_target)
    );
    assert_eq!(
        runtime.surface(id).unwrap().hovered_element(),
        Some(old_target)
    );

    runtime
        .update_surface(original, |staged| {
            staged
                .replace_root(root("advanced", new_element))
                .map(|_| ())
        })
        .unwrap();
    let advanced = runtime.surface_ref(id).unwrap();
    assert_ne!(advanced, original);
    let current_target = SurfaceElementRef::new(advanced, new_element);
    runtime.set_focus(advanced, Some(current_target)).unwrap();
    runtime.set_hover(advanced, Some(current_target)).unwrap();

    let result = runtime.update_surface(advanced, |staged| {
        *staged = escaped.take().unwrap();
        Ok(())
    });
    if result.is_ok() {
        let current = runtime.surface_ref(id).unwrap();
        assert_eq!(current.surface_id(), id);
        assert!(
            current.generation() > advanced.generation(),
            "accepting an escaped registry replacement must advance beyond the current generation"
        );
        let surface = runtime.surface(id).unwrap();
        assert_eq!(surface.root().id(), &RootId::new("original"));
        assert!(surface.root().elements().get(old_element).is_some());
        assert!(surface.root().elements().get(new_element).is_none());
        assert_ne!(surface.focused_element(), Some(old_target));
        assert_ne!(surface.hovered_element(), Some(old_target));
        assert_eq!(
            runtime
                .set_focus(advanced, Some(current_target))
                .unwrap_err()
                .code(),
            SurfaceErrorCode::StaleSurfaceGeneration
        );
    } else {
        assert_eq!(runtime.surface_ref(id), Some(advanced));
        let surface = runtime.surface(id).unwrap();
        assert_eq!(surface.root().id(), &RootId::new("advanced"));
        assert_eq!(surface.window_id(), window);
        assert!(surface.root().elements().get(old_element).is_none());
        assert!(surface.root().elements().get(new_element).is_some());
        assert_eq!(surface.focused_element(), Some(current_target));
        assert_eq!(surface.hovered_element(), Some(current_target));
        assert!(
            !runtime
                .set_focus(advanced, Some(current_target))
                .unwrap()
                .changed()
        );
        assert!(
            !runtime
                .set_hover(advanced, Some(current_target))
                .unwrap()
                .changed()
        );
    }
    assert_eq!(
        runtime
            .set_focus(original, Some(old_target))
            .unwrap_err()
            .code(),
        SurfaceErrorCode::StaleSurfaceGeneration
    );
    assert_eq!(
        runtime
            .set_hover(original, Some(old_target))
            .unwrap_err()
            .code(),
        SurfaceErrorCode::StaleSurfaceGeneration
    );
}

#[test]
fn whole_surface_replacement_cannot_reuse_the_old_registry_reference() {
    let id = SurfaceId::from_u64(37);
    let window = WindowId::from_u64(91);
    let old_element = ElementId::from_u64(1);
    let new_element = ElementId::from_u64(2);
    let mut runtime = Runtime::<()>::new((), ());
    let original = runtime
        .register_surface(UiSurface::try_new(id, window, root("original", old_element)).unwrap())
        .unwrap();
    let old_target = SurfaceElementRef::new(original, old_element);
    runtime.set_focus(original, Some(old_target)).unwrap();
    runtime.set_hover(original, Some(old_target)).unwrap();

    let result = runtime.update_surface(original, |current| {
        *current = UiSurface::try_new(id, window, root("replacement", new_element))?;
        Ok(())
    });

    if result.is_ok() {
        let current = runtime.surface_ref(id).unwrap();
        assert_eq!(current.surface_id(), id);
        assert_ne!(
            current, original,
            "an accepted registry replacement must invalidate the old surface reference"
        );
        let surface = runtime.surface(id).unwrap();
        assert_eq!(surface.root().id(), &RootId::new("replacement"));
        assert!(surface.root().elements().get(old_element).is_none());
        assert!(surface.root().elements().get(new_element).is_some());
        assert_eq!(
            runtime
                .set_focus(original, Some(old_target))
                .unwrap_err()
                .code(),
            SurfaceErrorCode::StaleSurfaceGeneration
        );
    } else {
        // Rejecting whole-value substitution is also valid, but failure must
        // preserve the original registration and its still-valid target.
        assert_eq!(runtime.surface_ref(id), Some(original));
        let surface = runtime.surface(id).unwrap();
        assert_eq!(surface.root().id(), &RootId::new("original"));
        assert_eq!(surface.window_id(), window);
        assert!(surface.root().elements().get(old_element).is_some());
        assert!(surface.root().elements().get(new_element).is_none());
        assert_eq!(surface.focused_element(), Some(old_target));
        assert_eq!(surface.hovered_element(), Some(old_target));
        assert!(
            !runtime
                .set_focus(original, Some(old_target))
                .unwrap()
                .changed()
        );
        assert!(
            !runtime
                .set_hover(original, Some(old_target))
                .unwrap()
                .changed()
        );
    }
}
