//! Counts below follow Ahem's one rectangular glyph: four provider path points,
//! Move + three Lines + Close, one foreground paint, and one original input.
use surgeist_render::*;
const FONT: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
fn input() -> (FontData, TextInstance, [TextGlyph; 1]) {
    (
        FontData::try_from_bytes(FONT.to_vec(), 0, surgeist_test::fixtures::FONT_LIMITS).unwrap(),
        TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap(),
        [TextGlyph::try_new(58, 0.0, 0.0, 1000.0).unwrap()],
    )
}
fn legacy() -> GlyphRunLimits {
    GlyphRunLimits::new(1, 5, 1 << 20, 1 << 20)
}
#[test]
fn real_glyph_has_one_semantic_input_and_separate_exact_traversal_counts() {
    let (font, instance, glyphs) = input();
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    );
    let output = font
        .prepare_glyph_run(&instance, &glyphs, legacy(), &mut context)
        .unwrap();
    assert_eq!(output.path_elements().count(), 5);
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.kind(), RenderAttemptKind::GlyphRun);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(report.usage().work(RenderWork::GlyphInputs), 1);
    assert_eq!(report.usage().work(RenderWork::CoordinateInputs), 0);
    assert_eq!(report.usage().work(RenderWork::PathElementsEmitted), 5);
    assert_eq!(report.usage().work(RenderWork::PaintVisits), 1);
    // n directory records and all n*(n-1)/2 duplicate comparisons, then
    // five complete searches for absent sbix/CBLC/CBDT/EBLC/EBDT tables.
    let records = u64::from(u16::from_be_bytes([FONT[4], FONT[5]]));
    assert_eq!(
        report.usage().work(RenderWork::FontRecordVisits),
        records * (records + 1) / 2 + 5 * records
    );
    assert_eq!(
        report
            .phase_usage(RenderPhase::Outline)
            .work(RenderWork::OutlinePointVisits),
        4
    );
    assert_eq!(
        report
            .phase_usage(RenderPhase::Outline)
            .work(RenderWork::PathElementVisits),
        5
    );
    assert_eq!(report.usage().work(RenderWork::GlyphVisits), 4); // input validation, dispatch, two raw-glyph qualifier loads
    assert_eq!(report.phase_count(RenderPhase::GlyphPrepare), 1);
    assert_eq!(report.phase_count(RenderPhase::Outline), 1);
    assert_eq!(report.phase_count(RenderPhase::Qualify), 2);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert!(report.usage().work(RenderWork::PreparedTransferredBytes) > 0);
}

#[test]
fn coalesced_foreground_inspection_is_a_visit_without_a_second_retained_paint() {
    let (font, instance, [glyph]) = input();
    let mut context = RenderAccountingContext::default();
    let output = font
        .prepare_glyph_run(
            &instance,
            &[glyph, glyph],
            GlyphRunLimits::new(2, 10, 1 << 20, 1 << 20),
            &mut context,
        )
        .unwrap();
    assert_eq!(output.path_elements().count(), 10);
    // One newly retained foreground paint, then one actual inspection of it
    // when the next glyph coalesces into its existing fill.
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::PaintVisits),
        2
    );
}

#[test]
fn bitmap_selection_refusal_preserves_the_typed_general_limit_and_accepted_prefix() {
    let (font, instance, glyphs) = input();
    let records = u64::from(u16::from_be_bytes([FONT[4], FONT[5]]));
    let directory = records * (records + 1) / 2;
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::FontRecordVisits, directory)
        .unwrap();
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let error = font
        .prepare_glyph_run(&instance, &glyphs, legacy(), &mut context)
        .unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(_))
    ));
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.usage().work(RenderWork::FontRecordVisits), directory);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    context.set_options(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::FontRecordVisits, directory + 5 * records)
                .unwrap(),
        ),
    );
    assert_eq!(
        font.prepare_glyph_run(&instance, &glyphs, legacy(), &mut context)
            .unwrap()
            .path_elements()
            .count(),
        5
    );
}
#[test]
fn one_under_general_and_legacy_limits_refuse_then_retry_the_same_input() {
    let (font, instance, glyphs) = input();
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::PathElementsEmitted, 4)
                .unwrap(),
        ),
    );
    let error = font
        .prepare_glyph_run(&instance, &glyphs, legacy(), &mut context)
        .unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(_))
    ));
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.usage().work(RenderWork::PathElementsEmitted), 4);
    assert_eq!(report.refusal().unwrap().phase(), RenderPhase::Attempt);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    context.set_options(RenderAccountingOptions::default());
    let error = font
        .prepare_glyph_run(
            &instance,
            &glyphs,
            GlyphRunLimits::new(1, 4, 1 << 20, 1 << 20),
            &mut context,
        )
        .unwrap_err();
    assert!(matches!(
        error.glyph_preparation_diagnostic(),
        Some(GlyphPreparationError::Limit {
            resource: GlyphResource::PathElements,
            required: 5,
            limit: 4
        })
    ));
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .refusal()
            .unwrap()
            .phase(),
        RenderPhase::Preparation
    );
    assert_eq!(
        font.prepare_glyph_run(&instance, &glyphs, legacy(), &mut context)
            .unwrap()
            .path_elements()
            .count(),
        5
    );
    assert_eq!(
        context.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
}
#[test]
fn malformed_input_retains_failed_prefix_without_transferring_result_custody() {
    let (font, instance, _) = input();
    let glyphs = [TextGlyph::try_new(u32::MAX, 0.0, 0.0, 0.0).unwrap()];
    let mut context = RenderAccountingContext::default();
    assert!(
        font.prepare_glyph_run(&instance, &glyphs, legacy(), &mut context)
            .is_err()
    );
    let report = context.take_last_attempt().unwrap();
    assert!(context.last_attempt().is_none());
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(ErrorCode::GlyphPreparation)
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::GlyphInputs),
        1
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::PreparedTransferredBytes),
        0
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        0
    );
}
#[test]
fn bounded_detail_loss_cannot_change_real_preparation_totals() {
    let (font, instance, glyphs) = input();
    let mut off = RenderAccountingContext::default();
    let mut bounded = RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(
        RenderProfile::Detailed {
            timing: false,
            event_limit: 1,
            filter: RenderPhaseFilter::All,
        },
    ));
    font.prepare_glyph_run(&instance, &glyphs, legacy(), &mut off)
        .unwrap();
    font.prepare_glyph_run(&instance, &glyphs, legacy(), &mut bounded)
        .unwrap();
    assert_eq!(
        bounded.last_attempt().unwrap().attempt().unwrap().usage(),
        off.last_attempt().unwrap().attempt().unwrap().usage()
    );
    assert_eq!(
        bounded
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .events()
            .count(),
        1
    );
    assert!(
        bounded
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .dropped_events()
            > 0
    );
}
