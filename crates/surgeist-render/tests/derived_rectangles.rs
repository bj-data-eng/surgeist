#![forbid(unsafe_code)]

use surgeist_render::{
    BackgroundAreas, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxDecorationBreak,
    BoxDecorationFragment, BoxDecorationInput, Capabilities, Color, Error, ErrorCode,
    ImagePlacementInput, ImageRepeatPlan, NormalizedBoxDecorationCommandKind, Outline,
    OutlineStyle, Radii, Rect, RenderAccountingContext, ResolvedImagePlacement, Scene, Size,
};

fn assert_invalid_numeric_input(error: Error) {
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    assert!(error.invalid_value_diagnostic().is_some());
    assert!(error.geometry_realization_diagnostic().is_none());
}

fn assert_overflowing_placement_is_rejected(intrinsic_size: Size, position: BackgroundPosition) {
    let input = ImagePlacementInput::try_new(
        Rect::try_new(0.0, 0.0, 1.0, 1.0).unwrap(),
        intrinsic_size,
        position,
        BackgroundSize::auto(),
    )
    .unwrap();
    let result = input.resolve();

    if let Ok(placement) = &result {
        let tile = placement.tile_rect();
        eprintln!(
            "published placement tile: {tile:?}, maximum: {:?}",
            tile.max()
        );
        // Exercise the existing public consumer if resolution publishes a tile.
        // A producer refusal must prevent this invalid rectangle from escaping.
        let mut scene = Scene::new();
        scene.clip(tile, |_| {});
    }

    assert_invalid_numeric_input(
        result.expect_err("placement must reject an overflowing maximum before publication"),
    );
}

#[test]
fn image_placement_rejects_overflowing_x_maximum_before_clipping() {
    // Every authored component is finite; the automatic tile would have
    // x = MAX and width = MAX, so its derived maximum cannot be finite.
    assert_overflowing_placement_is_rejected(
        Size::try_new(f64::MAX, 1.0).unwrap(),
        BackgroundPosition::length(f64::MAX, 0.0).unwrap(),
    );
}

#[test]
fn image_placement_rejects_overflowing_y_maximum_before_clipping() {
    assert_overflowing_placement_is_rejected(
        Size::try_new(1.0, f64::MAX).unwrap(),
        BackgroundPosition::length(0.0, f64::MAX).unwrap(),
    );
}

fn assert_overflowing_outline_is_rejected(border_box: Rect) {
    let areas = BackgroundAreas::try_new(border_box, border_box, border_box).unwrap();
    let fragment = BoxDecorationFragment::try_new(
        areas,
        Radii::try_all(0.0).unwrap(),
        BoxDecorationBreak::Slice,
    )
    .unwrap();
    let outline = Outline::try_new(OutlineStyle::Solid, 1.0, Color::BLACK, f64::MAX / 8.0).unwrap();
    let input = BoxDecorationInput::try_new(None, Some(outline), vec![fragment]).unwrap();
    let result = input.normalize(
        Capabilities::CURRENT,
        &mut RenderAccountingContext::default(),
    );

    if let Ok(normalized) = &result {
        for command in normalized.commands() {
            if let NormalizedBoxDecorationCommandKind::Outline(outline) = command.kind() {
                let target = outline.target_rect();
                eprintln!(
                    "published outline target: {target:?}, maximum: {:?}",
                    target.max()
                );
            }
        }
    }

    // A finite replacement or a suppressed command is also an incorrect success.
    assert_invalid_numeric_input(
        result.expect_err("outline must reject an overflowing maximum before publication"),
    );
}

#[test]
fn outline_rejects_overflowing_x_maximum_before_publication() {
    // The border box has maximum MAX. The offset produces finite x = 3MAX/8
    // and width = 3MAX/4, whose sum exceeds the finite f64 range.
    assert_overflowing_outline_is_rejected(
        Rect::try_new(f64::MAX / 2.0, 0.0, f64::MAX / 2.0, 1.0).unwrap(),
    );
}

#[test]
fn outline_rejects_overflowing_y_maximum_before_publication() {
    assert_overflowing_outline_is_rejected(
        Rect::try_new(0.0, f64::MAX / 2.0, 1.0, f64::MAX / 2.0).unwrap(),
    );
}

fn assert_overflowing_repeat_is_rejected(
    paint_rect: Rect,
    tile_rect: Rect,
    repeat: BackgroundRepeat,
) {
    let placement = ResolvedImagePlacement::from_parts(paint_rect, tile_rect).unwrap();
    let plan = ImageRepeatPlan::try_new(repeat, Capabilities::CURRENT).unwrap();
    let result = plan.resolve(placement, &mut RenderAccountingContext::default());

    if let Ok(resolved) = &result {
        for tile in resolved.tile_rects() {
            eprintln!("published repeat tile: {tile:?}, maximum: {:?}", tile.max());
        }
    }

    assert_invalid_numeric_input(
        result.expect_err("repeat must reject an overflowing tile maximum before publication"),
    );
}

#[test]
fn image_repeat_rejects_overflowing_second_x_tile_before_publication() {
    // Both supplied rectangles have finite maxima. Repetition adds a second
    // tile at approximately 3MAX/4 with the same width, overflowing its maximum.
    assert_overflowing_repeat_is_rejected(
        Rect::try_new(f64::MAX / 2.0, 0.0, f64::MAX / 2.0, 1.0).unwrap(),
        Rect::try_new(0.0, 0.0, f64::MAX / 4.0 * 3.0, 1.0).unwrap(),
        BackgroundRepeat::repeat_x(),
    );
}

#[test]
fn image_repeat_rejects_overflowing_second_y_tile_before_publication() {
    assert_overflowing_repeat_is_rejected(
        Rect::try_new(0.0, f64::MAX / 2.0, 1.0, f64::MAX / 2.0).unwrap(),
        Rect::try_new(0.0, 0.0, 1.0, f64::MAX / 4.0 * 3.0).unwrap(),
        BackgroundRepeat::repeat_y(),
    );
}
