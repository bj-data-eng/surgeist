#![forbid(unsafe_code)]

use surgeist_render::{
    BackgroundLayer, BackgroundNormalizationInput, BackgroundStack, ImageId, ResolvedImageResource,
    StyleImageLayer, StyleImageSource,
};

use surgeist_render::{
    BackgroundAreas, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxDecorationBreak,
    BoxDecorationFragment, BoxDecorationInput, Capabilities, Color, Error, ErrorCode,
    ImagePlacementInput, ImageRepeatPlan, NormalizedBoxDecorationCommandKind, Outline,
    OutlineStyle, Radii, Rect, RenderAccountingContext, RenderAccountingOptions,
    RenderAttemptStatus, RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile,
    RenderResource, RenderWork, ResolvedImagePlacement, Scene, Size,
};

fn assert_invalid_numeric_input(error: Error) {
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    let diagnostic = error.invalid_value_diagnostic().unwrap();
    assert!(diagnostic.field().contains(" max "));
    assert_eq!(diagnostic.value(), "inf");
    assert_eq!(diagnostic.invariant(), "must be finite");
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

fn profiles() -> [RenderProfile; 5] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 64,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::Phase(RenderPhase::Repeat),
        },
    ]
}

fn assert_clean_numeric_failure(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
    );
    assert_eq!(report.refusal(), None);
    let usage = report.usage();
    assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
}

fn ordinary_placement() -> ResolvedImagePlacement {
    ResolvedImagePlacement::from_parts(
        Rect::try_new(0.0, 0.0, 4.0, 4.0).unwrap(),
        Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(),
    )
    .unwrap()
}

#[test]
fn repeat_numeric_failure_keeps_one_emission_releases_backing_and_allows_retry() {
    for profile in profiles() {
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(profile));
        let placement = ResolvedImagePlacement::from_parts(
            Rect::try_new(f64::MAX / 2.0, 0.0, f64::MAX / 2.0, 1.0).unwrap(),
            Rect::try_new(0.0, 0.0, f64::MAX / 4.0 * 3.0, 1.0).unwrap(),
        )
        .unwrap();
        let repeat_x =
            ImageRepeatPlan::try_new(BackgroundRepeat::repeat_x(), Capabilities::CURRENT).unwrap();
        assert_invalid_numeric_input(repeat_x.resolve(placement, &mut context).unwrap_err());
        assert_clean_numeric_failure(&context);
        let report = context.last_attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::RepeatTiles), 1);
        // Three generated positions plus two maximum-coordinate proofs per
        // attempted tile: one valid tile, then one invalid tile.
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 7);
        assert_eq!(
            report
                .phase_usage(RenderPhase::Repeat)
                .work(RenderWork::CoordinateVisits),
            7
        );
        let repeat =
            ImageRepeatPlan::try_new(BackgroundRepeat::repeat(), Capabilities::CURRENT).unwrap();
        let output = repeat.resolve(ordinary_placement(), &mut context).unwrap();
        assert_eq!(
            output.tile_rects(),
            &[
                Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(),
                Rect::try_new(2.0, 0.0, 2.0, 2.0).unwrap(),
                Rect::try_new(0.0, 2.0, 2.0, 2.0).unwrap(),
                Rect::try_new(2.0, 2.0, 2.0, 2.0).unwrap(),
            ]
        );
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::RepeatTiles), 4);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 12);
        assert!(usage.work(RenderWork::PreparedTransferredBytes) > 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    }
}

fn outline_input(rects: &[Rect], offset: f64) -> BoxDecorationInput {
    let fragments = rects
        .iter()
        .map(|rect| {
            BoxDecorationFragment::try_new(
                BackgroundAreas::try_new(*rect, *rect, *rect).unwrap(),
                Radii::try_all(0.0).unwrap(),
                BoxDecorationBreak::Slice,
            )
            .unwrap()
        })
        .collect();
    BoxDecorationInput::try_new(
        None,
        Some(Outline::try_new(OutlineStyle::Solid, 1.0, Color::BLACK, offset).unwrap()),
        fragments,
    )
    .unwrap()
}

#[test]
fn outline_failure_preserves_first_command_work_and_retry_returns_exact_bounds() {
    let valid = Rect::try_new(10.0, 20.0, 30.0, 40.0).unwrap();
    let extreme = Rect::try_new(f64::MAX / 2.0, 0.0, f64::MAX / 2.0, 1.0).unwrap();
    let input = outline_input(&[valid, extreme], f64::MAX / 8.0);
    for profile in profiles() {
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(profile));
        assert_invalid_numeric_input(
            input
                .normalize(Capabilities::CURRENT, &mut context)
                .unwrap_err(),
        );
        assert_clean_numeric_failure(&context);
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CommandsEmitted), 1);
        // Each target derives four components and proves its two maxima.
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 12);
        let output = outline_input(&[valid], 2.0)
            .normalize(Capabilities::CURRENT, &mut context)
            .unwrap();
        assert_eq!(output.commands().len(), 1);
        let NormalizedBoxDecorationCommandKind::Outline(outline) = output.commands()[0].kind()
        else {
            panic!("expected an outline command")
        };
        assert_eq!(
            outline.target_rect(),
            Rect::try_new(8.0, 18.0, 34.0, 44.0).unwrap()
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            6
        );
    }
}

#[test]
fn repeat_maximum_proof_limit_refuses_before_emitting_unproved_tile() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CoordinateVisits, 5)
        .unwrap();
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let repeat =
        ImageRepeatPlan::try_new(BackgroundRepeat::repeat(), Capabilities::CURRENT).unwrap();
    let error = repeat
        .resolve(ordinary_placement(), &mut context)
        .unwrap_err();
    assert!(error.accounting_diagnostic().is_some());
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 4);
    assert_eq!(report.usage().work(RenderWork::RepeatTiles), 0);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn ordinary_placement_remains_exact_and_can_be_clipped() {
    let input = ImagePlacementInput::try_new(
        Rect::try_new(10.0, 20.0, 100.0, 80.0).unwrap(),
        Size::try_new(8.0, 6.0).unwrap(),
        BackgroundPosition::length(3.0, 4.0).unwrap(),
        BackgroundSize::auto(),
    )
    .unwrap();
    let resolved = input.resolve().unwrap();
    assert_eq!(
        resolved.tile_rect(),
        Rect::try_new(13.0, 24.0, 8.0, 6.0).unwrap()
    );
    let mut scene = Scene::new();
    scene.clip(resolved.tile_rect(), |_| {});
    assert_eq!(scene.len(), 1);
}

fn background_input(size: Size, position: BackgroundPosition) -> BackgroundNormalizationInput {
    let resource = ResolvedImageResource::try_new(ImageId::new(1), size).unwrap();
    let layer = StyleImageLayer::try_new(StyleImageSource::resolved(resource))
        .unwrap()
        .with_position(position)
        .with_repeat(BackgroundRepeat::no_repeat());
    let stack = BackgroundStack::try_new(None, vec![BackgroundLayer::new(layer)]).unwrap();
    let rect = Rect::try_new(0.0, 0.0, 1.0, 1.0).unwrap();
    BackgroundNormalizationInput::try_new(
        stack,
        BackgroundAreas::try_new(rect, rect, rect).unwrap(),
    )
    .unwrap()
}

#[test]
fn background_placement_failure_accounts_maximum_proof_without_emitting_output() {
    let invalid = background_input(
        Size::try_new(f64::MAX, 1.0).unwrap(),
        BackgroundPosition::length(f64::MAX, 0.0).unwrap(),
    );
    let valid = background_input(
        Size::try_new(1.0, 1.0).unwrap(),
        BackgroundPosition::length(0.0, 0.0).unwrap(),
    );
    for profile in profiles() {
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(profile));
        assert_invalid_numeric_input(
            invalid
                .normalize(Capabilities::CURRENT, &mut context)
                .unwrap_err(),
        );
        assert_clean_numeric_failure(&context);
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 2);
        assert_eq!(usage.work(RenderWork::RepeatTiles), 0);
        assert_eq!(usage.work(RenderWork::CommandsEmitted), 0);
        assert_eq!(
            valid
                .normalize(Capabilities::CURRENT, &mut context)
                .unwrap()
                .commands()
                .len(),
            1
        );
        let usage = context.last_attempt().unwrap().usage();
        // Placement proof2, no-repeat positions2, repeat-tile proof2.
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 6);
        assert_eq!(usage.work(RenderWork::RepeatTiles), 1);
        assert_eq!(usage.work(RenderWork::CommandsEmitted), 1);
    }
}
