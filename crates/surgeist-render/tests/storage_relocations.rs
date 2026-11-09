use std::time::Duration;
use surgeist_render::*;

const FONT: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn input() -> (FontData, TextInstance, Vec<TextGlyph>) {
    let font =
        FontData::try_from_bytes(FONT.to_vec(), 0, surgeist_test::fixtures::FONT_LIMITS).unwrap();
    let instance = TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap();
    let glyphs = (0..8)
        .map(|i| TextGlyph::try_new(58, f64::from(i) * 1000.0, 0.0, 1000.0).unwrap())
        .collect();
    (font, instance, glyphs)
}

fn caps() -> GlyphRunLimits {
    // Ahem has one rectangular outline: Move, three Lines and Close per glyph.
    GlyphRunLimits::new(8, 40, 1 << 20, 1 << 20)
}

#[test]
fn zero_relocation_cap_refuses_real_glyph_growth_and_allows_clean_retry() {
    let (font, instance, glyphs) = input();
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::StorageRelocations, 0)
        .unwrap();
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let error = font
        .prepare_glyph_run(&instance, &glyphs, caps(), &mut context)
        .unwrap_err();
    let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
        panic!("expected relocation refusal, got {error:?}");
    };
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::StorageRelocations)
    );
    assert_eq!(refusal.current(), 0);
    assert!(refusal.amount() > 0);
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.usage().work(RenderWork::StorageRelocations), 0);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);

    context.set_options(RenderAccountingOptions::new());
    let prepared = font
        .prepare_glyph_run(&instance, &glyphs, caps(), &mut context)
        .unwrap();
    assert_eq!(prepared.path_elements().count(), 40);
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert!(report.usage().work(RenderWork::StorageRelocations) > 0);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
}

fn clock() -> Duration {
    Duration::from_secs(1)
}

#[test]
fn glyph_relocation_usage_and_geometry_are_independent_of_profile_detail() {
    let (font, instance, glyphs) = input();
    let mut expected = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 1,
            filter: RenderPhaseFilter::Phase(RenderPhase::Outline),
        },
    ] {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(clock)),
        );
        let prepared = font
            .prepare_glyph_run(&instance, &glyphs, caps(), &mut context)
            .unwrap();
        let path: Vec<_> = prepared.path_elements().collect();
        assert_eq!(path.len(), 40);
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert!(report.usage().work(RenderWork::StorageRelocations) > 0);
        if let Some(expected) = &expected {
            assert_eq!(&(path, report.usage()), expected);
        } else {
            expected = Some((path, report.usage()));
        }
        if matches!(profile, RenderProfile::Detailed { event_limit: 0, .. }) {
            assert_eq!(report.events().count(), 0);
            assert!(report.dropped_events() > 0);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn renderer_rejects_a_forward_relocation_cap_before_native_initialization() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::StorageRelocations, 0)
        .unwrap();
    let result = pollster::block_on(Renderer::new(
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ));
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("CPU lifetime relocations have no forward frame producer"),
    };
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::InvalidLimit(
            RenderWork::StorageRelocations
        ))
    );
}
