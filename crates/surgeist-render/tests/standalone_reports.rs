#![forbid(unsafe_code)]

use surgeist_render::{
    Color, Rect, RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind,
    RenderAttemptStatus, RenderLimits, RenderPhaseFilter, RenderProfile, RenderResource,
    RenderStandaloneReport, RenderWork, Scene,
};

fn scene() -> Scene {
    let mut scene = Scene::new();
    scene.fill(Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(), Color::BLACK);
    scene
}

fn profiles() -> [RenderProfile; 5] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 8,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
    ]
}

#[test]
fn taken_terminal_record_retains_its_report_after_context_destruction() {
    for profile in profiles() {
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(profile));
        let summary = scene().inspect(&mut context).unwrap();
        assert_eq!(summary.commands, 1);
        let record: RenderStandaloneReport = context.take_last_attempt().unwrap();
        assert!(context.last_attempt().is_none());
        drop(context);

        assert_eq!(record.kind(), RenderAttemptKind::SceneInspection);
        assert_eq!(record.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(record.cleanup_fault(), None);
        let report = record.attempt().unwrap();
        assert_eq!(report.kind(), record.kind());
        assert_eq!(report.status(), record.status());
        assert_eq!(report.usage().work(RenderWork::AuthoredCommands), 1);
        assert_eq!(report.usage().work(RenderWork::AuthoredFills), 1);
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    }
}

#[test]
fn public_refusal_record_survives_a_successful_retry_on_the_same_context() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::AuthoredCommands, 0)
        .unwrap();
    for profile in profiles() {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_profile(profile)
                .with_limits(limits),
        );
        let error = scene().inspect(&mut context).unwrap_err();
        let refused = context.take_last_attempt().unwrap();
        assert_eq!(refused.status(), RenderAttemptStatus::Refused);
        assert_eq!(refused.cleanup_fault(), None);
        let report = refused.attempt().unwrap();
        assert!(error.accounting_diagnostic().is_some());
        let refusal = report.refusal().unwrap();
        assert_eq!(refusal.current(), 0);
        assert_eq!(refusal.amount(), 1);
        assert_eq!(report.usage().work(RenderWork::AuthoredCommands), 0);
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);

        context.set_options(RenderAccountingOptions::new().with_profile(profile));
        assert_eq!(scene().inspect(&mut context).unwrap().commands, 1);
        assert_eq!(
            context.last_attempt().unwrap().status(),
            RenderAttemptStatus::Succeeded
        );
        assert_eq!(refused.status(), RenderAttemptStatus::Refused);
        assert_eq!(refused.attempt().unwrap().refusal(), Some(refusal));
    }
}
