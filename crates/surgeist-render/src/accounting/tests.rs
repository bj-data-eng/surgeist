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

// These fixtures deliberately misuse Ledger's public custody operations. Public
// Render callers cannot access the operation or release an owned hold early.
fn release_preparation_custody_before_hold_drop(account: &Account) {
    let resource = RenderResource::RetainedCpuBytes;
    let scope = account.scopes[RenderPhase::Preparation.index()];
    let hold = account.acquire(resource, 1).unwrap();
    account
        .ledger
        .apply(scope, Change::Release(resource, 1))
        .unwrap();
    // Establish the real public-Ledger refusal the actual destructor will meet.
    assert!(matches!(
        account.ledger.check_batch(scope, &[Change::Release(resource, 1)]),
        Err(LedgerError::Refused(refusal))
            if refusal.current == 0
                && refusal.change == Change::Release(resource, 1)
                && refusal.reason == surgeist_ledger::RefusalReason::Underflow
    ));
    drop(hold);
}

fn cancel_image_repeat_after_hold_underflow(context: &mut RenderAccountingContext) {
    let attempt = Attempt::begin(
        context,
        RenderAttemptKind::ImageRepeat,
        GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
        #[cfg(feature = "render-text")]
        None,
    )
    .unwrap();
    release_preparation_custody_before_hold_drop(&attempt.account);
    drop(attempt);
}

fn assert_latched_underflow(account: &Account, phase: RenderPhase, resource: RenderResource) {
    let Some(RenderAccountingError::Refused(refusal)) = account.latched_fault() else {
        panic!(
            "drop must retain the typed custody underflow: {:?}",
            account.latched_fault()
        );
    };
    assert_eq!(refusal.phase(), phase);
    assert_eq!(refusal.quantity(), RenderQuantity::Resource(resource));
    assert_eq!(refusal.current(), 0);
    assert_eq!(refusal.amount(), 1);
    assert_eq!(refusal.reason(), RenderRefusalReason::Underflow);
}

#[test]
fn drop_fault_cancellation_replaces_prior_attempt_report() {
    let mut context = RenderAccountingContext::default();
    crate::Scene::new().inspect(&mut context).unwrap();
    assert_eq!(
        context.last_attempt().unwrap().kind(),
        RenderAttemptKind::SceneInspection
    );

    cancel_image_repeat_after_hold_underflow(&mut context);

    let report = context.last_attempt().unwrap();
    assert_eq!(report.kind(), RenderAttemptKind::ImageRepeat);
    assert_eq!(report.status(), RenderAttemptStatus::Canceled);
}

#[test]
fn drop_fault_cancellation_does_not_fail_next_attempt() {
    let mut context = RenderAccountingContext::default();
    cancel_image_repeat_after_hold_underflow(&mut context);

    crate::Scene::new()
        .inspect(&mut context)
        .expect("a canceled attempt's drop fault must not fail an unrelated inspection");
    let report = context.last_attempt().unwrap();
    assert_eq!(report.kind(), RenderAttemptKind::SceneInspection);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
}

#[test]
fn drop_fault_preparation_hold_preserves_underflow_details() {
    let mut context = RenderAccountingContext::default();
    let attempt = Attempt::begin(
        &mut context,
        RenderAttemptKind::GlyphRun,
        GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
        #[cfg(feature = "render-text")]
        None,
    )
    .unwrap();
    release_preparation_custody_before_hold_drop(&attempt.account);
    assert_latched_underflow(
        &attempt.account,
        RenderPhase::Preparation,
        RenderResource::RetainedCpuBytes,
    );
}

fn account_with_child_below_repeat_quota() -> (Account, Scope) {
    // Render's ordinary fixed table is full and its quota scopes are leaves.
    // This valid Ledger fixture reserves one extra scope to exercise genuine
    // inclusive-versus-direct custody refusal; no Scope identity is fabricated.
    let mut operation =
        Operation::new("drop-fault-fixture", Limits::unlimited(), PHASE_COUNT + 1).unwrap();
    let mut scopes = [operation.root(); PHASE_COUNT];
    for phase in RenderPhase::ALL.into_iter().skip(1) {
        let limits = match phase {
            RenderPhase::RepeatQuota => Limits::new(
                RenderWorkTable::splat(u64::MAX),
                RenderResourceTable {
                    LocalRepeatTileQuota: 1_000_000,
                    ..RenderResourceTable::splat(u64::MAX)
                },
            ),
            RenderPhase::AtlasQuota => {
                let texels = u64::from(u16::MAX) * u64::from(u16::MAX);
                Limits::new(
                    RenderWorkTable::splat(u64::MAX),
                    RenderResourceTable {
                        LocalAtlasSourceByteQuota: texels * 4,
                        LocalAtlasTexelQuota: texels,
                        ..RenderResourceTable::splat(u64::MAX)
                    },
                )
            }
            _ => Limits::unlimited(),
        };
        scopes[phase.index()] = operation
            .child(
                scopes[phase.parent().unwrap().index()],
                phase.name(),
                limits,
                Tariffs::identity(),
            )
            .unwrap();
    }
    let child = operation
        .child(
            scopes[RenderPhase::RepeatQuota.index()],
            "actual-child-custody",
            Limits::unlimited(),
            Tariffs::identity(),
        )
        .unwrap();
    (
        Account {
            ledger: operation.share(),
            scopes: Arc::new(scopes),
            phase: RenderPhase::Preparation,
            cpu: None,
            history: None,
            custody_open: Arc::new(AtomicBool::new(true)),
            first_refusal: Arc::new(Mutex::new(None)),
            fault: Arc::new(Mutex::new(None)),
            kind: RenderAttemptKind::ImageRepeat,
        },
        child,
    )
}

#[test]
fn drop_fault_local_quota_preserves_underflow_details() {
    let (account, child) = account_with_child_below_repeat_quota();
    let resource = RenderResource::LocalRepeatTileQuota;
    let quota = account
        .local_quota(RenderPhase::Repeat, [1_000_000; 2])
        .unwrap();
    account
        .ledger
        .apply(child, Change::Acquire(resource, 1))
        .unwrap();
    let quota_scope = account.scopes[RenderPhase::RepeatQuota.index()];
    assert_eq!(account.ledger.usage(quota_scope).unwrap().live(resource), 1);
    // Ledger permits only the acquiring child to release this custody. The
    // actual quota destructor will attempt the parent's inclusive balance.
    assert!(matches!(
        account.ledger.check_batch(quota_scope, &[Change::Release(resource, 1)]),
        Err(LedgerError::Refused(refusal))
            if refusal.current == 0
                && refusal.change == Change::Release(resource, 1)
                && refusal.reason == surgeist_ledger::RefusalReason::Underflow
    ));

    drop(quota);

    // This fixture deliberately has an extra scope, so it observes the fault
    // latch directly and never projects a fixed-table Render attempt report.
    let fault = account.latched_fault();
    account
        .ledger
        .apply(child, Change::Release(resource, 1))
        .unwrap();
    assert_eq!(account.ledger.usage(quota_scope).unwrap().live(resource), 0);
    assert!(
        fault.is_some(),
        "quota drop discarded its real Ledger refusal"
    );
    assert_latched_underflow(&account, RenderPhase::RepeatQuota, resource);
}

#[test]
fn drop_fault_distinct_ledger_defects_remain_distinct() {
    // Valid Ledger sequencing cannot produce a Defect. This checks conversion
    // of public error values only, not execution of a fabricated broken ledger.
    assert_ne!(
        map_ledger_failure(LedgerError::Defect(surgeist_ledger::Defect::ScopeEntry)),
        map_ledger_failure(LedgerError::Defect(surgeist_ledger::Defect::ReportScope)),
    );
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
        RenderWork::ReadbackTransferredBytes,
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
fn standalone_inspection_rejects_a_renderer_readback_limit_without_replacing_its_report() {
    let scene = crate::Scene::new();
    let mut context = RenderAccountingContext::default();
    scene.inspect(&mut context).unwrap();
    let prior = context.last_attempt().unwrap().clone();
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ReadbackRows, 1)
        .expect("the renderer readback producer supports row admission");
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    assert_eq!(
        scene
            .inspect(&mut context)
            .unwrap_err()
            .accounting_diagnostic(),
        Some(RenderAccountingError::InvalidLimit(
            RenderWork::ReadbackRows
        )),
    );
    assert_eq!(context.last_attempt(), Some(&prior));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn renderer_setup_accepts_frame_and_readback_units_in_one_fixed_configuration() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ReadbackRows, 0)
        .expect("zero is a supported refusal limit for actual readback rows")
        .with_work_limit(RenderWork::ReadbackBytes, 0)
        .expect("logical row payload bytes have a readback producer")
        .with_work_limit(RenderWork::Submissions, 0)
        .unwrap();
    let renderer = pollster::block_on(crate::Renderer::new(
        crate::Options::default()
            .with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ));
    assert!(
        renderer.is_ok(),
        "fixed renderer options cover both operation fronts"
    );
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
                Ok(output.into_preparation()?.into_retained())
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

#[test]
fn reports_refuse_ledger_intervals_that_lack_the_fixed_scopes() {
    use surgeist_ledger::{IntervalReport, Limits, Operation};
    let operation =
        Operation::<RenderWork, RenderResource>::new("test", Limits::unlimited(), 1).unwrap();
    let identity = RenderCpuIdentity(operation.root());
    let empty = || IntervalReport {
        scopes: Vec::new(),
        profile: None,
    };
    assert_eq!(
        RenderAttemptReport::new(
            RenderAttemptKind::Frame,
            RenderAttemptStatus::Succeeded,
            empty(),
            None
        )
        .unwrap_err(),
        RenderAccountingError::Invariant
    );
    assert_eq!(
        RenderCpuInterval::new(identity, empty()).unwrap_err(),
        RenderAccountingError::Invariant
    );
}
