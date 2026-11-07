use super::*;

#[test]
fn ledger_failures_preserve_setup_causes_and_distinguish_private_invariants() {
    for (ledger, render) in [
        (
            LedgerError::Allocation,
            RenderAccountingSetupError::Allocation,
        ),
        (
            LedgerError::TimingUnavailable,
            RenderAccountingSetupError::TimingUnavailable,
        ),
        (
            LedgerError::IdentityExhausted,
            RenderAccountingSetupError::IdentityExhausted,
        ),
    ] {
        assert_eq!(
            map_ledger_failure(ledger),
            RenderAccountingError::Setup(render)
        );
    }
    for ledger in [
        LedgerError::ForeignScope,
        LedgerError::ForeignCheckpoint,
        LedgerError::StaleCheckpoint,
        LedgerError::CheckpointLimit { limit: 1 },
        LedgerError::CheckpointsConfigured,
        LedgerError::ScopeLimit { limit: 12 },
    ] {
        assert_eq!(map_ledger_failure(ledger), RenderAccountingError::Invariant);
    }
}
use std::sync::atomic::{AtomicU64, Ordering};
static TIME: AtomicU64 = AtomicU64::new(0);
fn clock() -> std::time::Duration {
    std::time::Duration::from_millis(TIME.load(Ordering::SeqCst))
}
#[test]
fn exact_growth_overlap_and_refused_preview_leave_old_backing_unchanged() {
    let mut context = RenderAccountingContext::default();
    context
        .prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, 7, 0),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let mut v = account.temporary::<u8>(3)?;
                v.extend_from_slice(&[1, 2, 3])?;
                v.push(4)?;
                assert_eq!(v.capacity(), 4);
                assert!(v.grow(1).is_err());
                assert_eq!(v.as_slice(), [1, 2, 3, 4]);
                Ok(())
            },
        )
        .unwrap();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 7);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
}
#[test]
fn ended_phase_timing_and_output_transfer_are_independent_of_detail_loss() {
    TIME.store(0, Ordering::SeqCst);
    let options = RenderAccountingOptions::new()
        .with_clock(RenderClock::Host(clock))
        .with_profile(RenderProfile::Detailed {
            timing: true,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        });
    let mut context = RenderAccountingContext::new(options);
    context
        .prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(1, 0, 0, 9),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                TIME.store(5, Ordering::SeqCst);
                account.phase(RenderPhase::Outline, || {
                    account.work(RenderWork::GlyphInputs, 1)?;
                    account.retain(9)?;
                    TIME.store(11, Ordering::SeqCst);
                    Ok(())
                })
            },
        )
        .unwrap();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 9);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(report.usage().peak(RenderResource::RetainedCpuBytes), 9);
    assert_eq!(
        report.phase_elapsed(RenderPhase::Outline),
        Some(std::time::Duration::from_millis(6))
    );
    assert_eq!(
        report.phase_elapsed(RenderPhase::Preparation),
        Some(std::time::Duration::from_millis(11))
    );
    assert_eq!(report.events().count(), 0);
    assert!(report.dropped_events() > 0);
}
#[test]
fn off_and_untimed_do_not_call_the_supplied_clock() {
    fn forbidden() -> std::time::Duration {
        panic!("untimed clock must never be read")
    }
    for profile in [RenderProfile::Off, RenderProfile::Summary { timing: false }] {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_clock(RenderClock::Host(forbidden))
                .with_profile(profile),
        );
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                GlyphRunLimits::new(1, 0, 0, 0),
                #[cfg(feature = "render-text")]
                None,
                |account| account.work(RenderWork::GlyphInputs, 1),
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .work(RenderWork::GlyphInputs),
            1
        );
        assert_eq!(context.last_attempt().unwrap().elapsed(), None);
    }
}
#[test]
fn unwinding_drops_backing_then_retains_canceled_report() {
    let mut context = RenderAccountingContext::default();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: Result<()> = context.prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(1, 0, 8, 8),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let _scratch = account.temporary::<u8>(8)?;
                account.retain(8)?;
                panic!("cancel preparation");
            },
        );
    }));
    assert!(panic.is_err());
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Canceled);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
}
#[test]
fn report_setup_failure_preserves_the_previous_completed_attempt() {
    let mut context = RenderAccountingContext::default();
    context
        .prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, 0, 0),
            #[cfg(feature = "render-text")]
            None,
            |_| Ok(()),
        )
        .unwrap();
    let previous = context.last_attempt().unwrap().clone();
    context.set_options(
        RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
            timing: false,
            event_limit: usize::MAX,
            filter: RenderPhaseFilter::All,
        }),
    );
    assert!(
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                GlyphRunLimits::new(0, 0, 0, 0),
                #[cfg(feature = "render-text")]
                None,
                |_| -> Result<()> { panic!("setup must fail first") }
            )
            .is_err()
    );
    assert_eq!(context.last_attempt(), Some(&previous));
}
#[test]
fn failed_allocator_reservation_does_not_create_a_live_peak() {
    let mut context = RenderAccountingContext::default();
    let result: Result<()> = context.prepare(
        RenderAttemptKind::GlyphRun,
        GlyphRunLimits::new(0, 0, usize::MAX, 0),
        #[cfg(feature = "render-text")]
        None,
        |account| {
            let _impossible = account.temporary::<u8>(usize::MAX)?;
            Ok(())
        },
    );
    assert!(matches!(
        result.unwrap_err().glyph_preparation_diagnostic(),
        Some(GlyphPreparationError::Allocation {
            resource: GlyphResource::ScratchBytes
        })
    ));
    let report = context.last_attempt().unwrap();
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::GlyphPreparation)
    );
    assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn standalone_setup_rejects_foreign_work_and_preserves_the_previous_report() {
    let scene = crate::Scene::new();
    let mut context = RenderAccountingContext::default();
    scene.inspect(&mut context).unwrap();
    let prior = context.last_attempt().unwrap().clone();
    // CommandsEmitted is meaningful for frame normalization, but an inspection
    // consumes authored records and never normalizes or emits commands.
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CommandsEmitted, 0)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    assert_eq!(
        scene
            .inspect(&mut context)
            .unwrap_err()
            .accounting_diagnostic(),
        Some(RenderAccountingError::InvalidLimit(
            RenderWork::CommandsEmitted
        ))
    );
    assert_eq!(context.last_attempt(), Some(&prior));
    for work in [
        RenderWork::ReadbackRows,
        RenderWork::ResourceRetirements,
        RenderWork::PreparedTransferredBytes,
    ] {
        assert_eq!(
            RenderLimits::new().with_work_limit(work, 0),
            Err(RenderAccountingError::InvalidLimit(work))
        );
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn renderer_setup_rejects_authored_inspection_units_before_initialization() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::AuthoredCommands, 0)
        .unwrap();
    let result = pollster::block_on(crate::Renderer::new(
        crate::Options::default()
            .with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ));
    let Err(error) = result else {
        panic!("frame options must reject an inspection-only unit");
    };
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::InvalidLimit(
            RenderWork::AuthoredCommands
        ))
    );
}

#[test]
fn retained_growth_admits_actual_overlap_and_transfers_only_final_backing() {
    for (limit, succeeds, peak, transferred) in [(6, false, 3, 0), (7, true, 7, 4)] {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(limit)),
        );
        let result = context.prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, 0, 100),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let mut output = account.retained::<u8>(3)?;
                output.extend_from_slice(&[1, 2, 3])?;
                output.push(4)?;
                Ok(output.into_retained())
            },
        );
        assert_eq!(result.is_ok(), succeeds);
        if let Ok(values) = result {
            assert_eq!(values, [1, 2, 3, 4]);
        }
        let report = context.last_attempt().unwrap();
        assert_eq!(report.usage().peak(RenderResource::RetainedCpuBytes), peak);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(
            report.usage().work(RenderWork::PreparedTransferredBytes),
            transferred
        );
        if !succeeds {
            assert_eq!(report.refusal().unwrap().phase(), RenderPhase::Attempt);
        }
    }
}
