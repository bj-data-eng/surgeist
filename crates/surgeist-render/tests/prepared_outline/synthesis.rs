//! Literal extracted-path synthesis facts from the pinned WebKit operation.
//! These are appended contours, not a boolean union or two alpha applications.
use super::*;

fn bold(size: f32, skew: f32) -> TextInstance {
    let displacement = size / 36.0_f32;
    assert!(displacement > 0.0);
    TextInstance::try_new(
        size,
        vec![],
        OutlineSynthesis::try_new(skew, displacement).unwrap(),
    )
    .unwrap()
}

fn xy(element: PathElement) -> Option<(f64, f64)> {
    match element {
        PathElement::MoveTo(point) | PathElement::LineTo(point) => Some((point.x(), point.y())),
        _ => None,
    }
}

#[test]
fn explicit_positive_bold_appends_two_literal_contours_without_changing_glyph_facts() {
    let font = font(AHEM);
    let instance = bold(360.0, 0.0);
    assert_eq!(instance.synthesis().embolden(), 10.0);
    for advance in [-123.0, 0.0] {
        let glyphs = [TextGlyph::try_new(X, 20.0, 30.0, advance).unwrap()];
        let prepared = font.prepare_outline(&instance, &glyphs, limits()).unwrap();
        assert_eq!(prepared.font(), &font);
        assert_eq!(prepared.instance(), &instance);
        assert_eq!(prepared.glyphs(), glyphs);
        // Ahem X: source (0,-200)..(1000,800), UPEM1000. Scale360,
        // then local (20,30); copy adds local +10, never +10*0.36.
        assert_eq!(
            prepared.bounds().ink_rect(),
            Some(Rect::try_new(20.0, -258.0, 370.0, 360.0).unwrap())
        );
        let path: Vec<_> = prepared.path_elements().collect();
        assert_eq!(path.len(), 10);
        assert!(matches!(path[0], PathElement::MoveTo(_)));
        assert!(matches!(path[5], PathElement::MoveTo(_)));
        assert!(matches!(path[4], PathElement::Close));
        assert!(matches!(path[9], PathElement::Close));
        let expected = [
            (20.0, -258.0),
            (380.0, -258.0),
            (380.0, 102.0),
            (20.0, 102.0),
            (30.0, -258.0),
            (390.0, -258.0),
            (390.0, 102.0),
            (30.0, 102.0),
        ];
        assert_eq!(
            path.into_iter().filter_map(xy).collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn explicit_bold_after_existing_x_shear_keeps_local_displacement_and_literal_bounds() {
    let font = font(AHEM);
    let instance = bold(360.0, 0.5);
    let glyphs = [TextGlyph::try_new(X, 20.0, 30.0, -10.0).unwrap()];
    let prepared = font.prepare_outline(&instance, &glyphs, limits()).unwrap();
    // x*s + k*y*s + gx, then +j*b. Top y800 contributes +144,
    // bottom y-200 contributes -36; neither copy adjusts y or advance.
    assert_eq!(
        prepared.bounds().ink_rect(),
        Some(Rect::try_new(-16.0, -258.0, 550.0, 360.0).unwrap())
    );
    let path: Vec<_> = prepared.path_elements().collect();
    assert_eq!(path.len(), 10);
    assert_eq!(xy(path[0]), Some((164.0, -258.0)));
    assert_eq!(xy(path[1]), Some((524.0, -258.0)));
    assert_eq!(xy(path[3]), Some((-16.0, 102.0)));
    assert_eq!(xy(path[5]), Some((174.0, -258.0)));
    assert_eq!(xy(path[6]), Some((534.0, -258.0)));
    assert_eq!(prepared.instance().synthesis(), instance.synthesis());
    assert_eq!(prepared.glyphs(), glyphs);
}

#[test]
fn positive_bold_preserves_source_proved_empty_glyphs_and_empty_input() {
    let font = font(AHEM);
    let instance = bold(360.0, 0.0);
    for glyphs in [
        vec![],
        vec![TextGlyph::try_new(3, 20.0, 30.0, 360.0).unwrap()],
        vec![TextGlyph::try_new(253, -50.0, -60.0, 0.0).unwrap()],
    ] {
        let prepared = font.prepare_outline(&instance, &glyphs, limits()).unwrap();
        assert_eq!(prepared.bounds().kind(), TextRunBoundsKind::Empty);
        assert_eq!(prepared.path_elements().count(), 0);
        assert_eq!(prepared.glyphs(), glyphs);
        assert_eq!(prepared.instance(), &instance);
    }
}

#[test]
fn bold_copy_uses_existing_limits_and_same_input_raised_retry_is_atomic() {
    let font = font(AHEM);
    let retained_font = font.clone();
    let instance = bold(360.0, 0.0);
    let retained_instance = instance.clone();
    let glyphs = [TextGlyph::try_new(X, 20.0, 30.0, -123.0).unwrap()];
    let mut failures = Vec::new();
    for (resource, low) in [
        (
            OutlineResource::Glyphs,
            OutlineLimits::new(0, 10, 1 << 20, 1 << 20),
        ),
        (
            OutlineResource::PathElements,
            OutlineLimits::new(1, 9, 1 << 20, 1 << 20),
        ),
        (
            OutlineResource::ScratchBytes,
            OutlineLimits::new(1, 10, 0, 1 << 20),
        ),
        (
            OutlineResource::RetainedBytes,
            OutlineLimits::new(1, 10, 1 << 20, 0),
        ),
    ] {
        match font.prepare_outline(&instance, &glyphs, low) {
            Err(error) if matches!(diagnostic(&error),OutlinePreparationError::Limit{resource:actual,..} if actual==resource) =>
                {}
            Err(error) => failures.push(format!("{resource:?}: {:?}", diagnostic(&error))),
            Ok(_) => failures.push(format!("{resource:?}: unexpected output")),
        }
        assert_eq!(font, retained_font);
        assert_eq!(instance, retained_instance);
        assert_eq!(glyphs[0].advance(), -123.0);
    }
    match font.prepare_outline(
        &instance,
        &glyphs,
        OutlineLimits::new(1, 10, 1 << 20, 1 << 20),
    ) {
        Ok(prepared) => {
            assert_eq!(prepared.path_elements().count(), 10);
            assert_eq!(
                prepared.bounds().ink_rect(),
                Some(Rect::try_new(20.0, -258.0, 370.0, 360.0).unwrap())
            );
            assert_eq!(prepared.font(), &retained_font);
            assert_eq!(prepared.instance(), &retained_instance);
            assert_eq!(prepared.glyphs(), glyphs);
        }
        Err(error) => failures.push(format!("same-input raised retry: {:?}", diagnostic(&error))),
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn positive_copy_displacement_lost_at_canonical_point_conversion_refuses_whole_run() {
    let font = font(AHEM);
    let far = TextGlyph::try_new(X, 33_554_432.0, 0.0, 0.0).unwrap();
    let plain = instance(36.0, vec![], 0.0);
    // Original X width36 survives f32 (spacing4 here); positive b=1 does
    // not move any canonical x. This isolates copy loss from original ink loss.
    let original = font.prepare_outline(&plain, &[far], limits()).unwrap();
    assert_eq!(original.bounds().ink_rect().unwrap().width(), 36.0);
    let bold = bold(36.0, 0.0);
    assert_eq!(bold.synthesis().embolden(), 1.0);
    let glyphs = [TextGlyph::try_new(X, 20.0, 30.0, -10.0).unwrap(), far];
    let kept_font = font.clone();
    let kept_instance = bold.clone();
    let error = font.prepare_outline(&bold, &glyphs, limits()).unwrap_err();
    assert!(
        matches!(diagnostic(&error), OutlinePreparationError::Numeric { .. }),
        "{:?}",
        diagnostic(&error)
    );
    assert_eq!(font, kept_font);
    assert_eq!(bold, kept_instance);
    assert_eq!(glyphs[1], far);
    assert_eq!(original.glyphs(), [far]);
}
