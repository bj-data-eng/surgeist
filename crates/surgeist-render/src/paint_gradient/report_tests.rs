//! Whole-record preservation through the public Paint integration front door.
use crate::{
    Color, RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
    RenderAccountingSetupError, RenderAttemptStatus, RenderLimits, RenderPhaseFilter,
    RenderProfile, RenderWork, prepare_repeating_linear_gradient,
};
use surgeist_layout as layout;
use surgeist_paint as paint;

fn red() -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
}

fn blue() -> Color {
    Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()
}

fn paint_options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}

fn position(length: f64, fraction: f64) -> paint::GradientPosition {
    paint::GradientPosition { length, fraction }
}

fn at(value: f64, color: Color) -> paint::GradientStopItem<Color> {
    paint::GradientStopItem::Stop {
        payload: color,
        positions: paint::GradientStopPositions::One(position(value, 0.0)),
    }
}

fn line(length: f64) -> paint::LinearGradientLine {
    paint::resolve_linear_gradient(
        layout::Size::<f64>::new(length, 1.0),
        paint::LinearGradientDirection::Right,
        paint_options(),
    )
    .result
    .unwrap()
}

fn stops(
    length: f64,
    source: &[paint::GradientStopItem<Color>],
) -> paint::GradientStops<'_, Color> {
    paint::resolve_gradient_stops(length, source, paint_options())
        .result
        .unwrap()
}

fn context(limits: RenderLimits) -> RenderAccountingContext {
    RenderAccountingContext::new(
        RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Summary { timing: false }),
    )
}

#[test]
fn invalid_limit_and_report_setup_failure_preserve_the_previous_completed_attempt() {
    let geometry = line(8.0);
    let source = [at(0.0, red()), at(8.0, blue())];
    let used = stops(8.0, &source);
    let mut account = context(RenderLimits::new());
    prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    let previous = account.last_attempt().unwrap().snapshot_for_test();
    account.set_options(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::GlyphVisits, 0)
                .unwrap(),
        ),
    );
    let error = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::InvalidLimit(RenderWork::GlyphVisits))
    ));
    assert_eq!(account.last_attempt(), Some(&previous));
    account.set_options(
        RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
            timing: false,
            event_limit: usize::MAX,
            filter: RenderPhaseFilter::All,
        }),
    );
    let error = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Setup(
            RenderAccountingSetupError::Allocation
        ))
    ));
    assert_eq!(account.last_attempt(), Some(&previous));
    account.set_options(RenderAccountingOptions::new());
    prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    assert_eq!(
        account.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
}
