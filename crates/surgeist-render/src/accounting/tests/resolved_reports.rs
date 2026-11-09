//! Adapter-boundary fixtures use actual Ledger handles and public report fields.
//! Malformed reports here are not outputs of valid Ledger sequencing and are not
//! public Render reproducers or fabricated provider defects.

use super::super::*;
use std::{cell::Cell, time::Duration};
use surgeist_ledger::{Event, IntervalReport, ProfileMode, Refusal, Report};

thread_local! {
    static NOW: Cell<u64> = const { Cell::new(0) };
}

fn clock() -> Duration {
    Duration::from_millis(NOW.get())
}

#[derive(Clone, Copy, Debug)]
enum Domain {
    Attempt,
    CpuInterval,
    CpuSnapshot,
    GenerationInterval,
    DeviceSnapshot,
}

impl Domain {
    fn scope_count(self) -> usize {
        match self {
            Self::Attempt => PHASE_COUNT,
            Self::CpuInterval | Self::CpuSnapshot => RenderCpuScope::ALL.len(),
            Self::GenerationInterval | Self::DeviceSnapshot => RenderGenerationScope::ALL.len(),
        }
    }

    fn parent(self, index: usize) -> usize {
        match self {
            Self::Attempt => RenderPhase::ALL[index].parent().unwrap().index(),
            Self::CpuInterval | Self::CpuSnapshot => RenderCpuScope::Root.index(),
            Self::GenerationInterval | Self::DeviceSnapshot => {
                match RenderGenerationScope::ALL[index] {
                    RenderGenerationScope::Vello
                    | RenderGenerationScope::Effects
                    | RenderGenerationScope::Masks
                    | RenderGenerationScope::Kernels => RenderGenerationScope::Cache.index(),
                    RenderGenerationScope::SurfaceTargets
                    | RenderGenerationScope::PassBuffers
                    | RenderGenerationScope::ReadbackStaging => {
                        RenderGenerationScope::External.index()
                    }
                    RenderGenerationScope::ShaderModule
                    | RenderGenerationScope::BindGroupLayout
                    | RenderGenerationScope::PipelineLayout
                    | RenderGenerationScope::ComputePipeline
                    | RenderGenerationScope::RenderPipeline
                    | RenderGenerationScope::Sampler
                    | RenderGenerationScope::BindGroup
                    | RenderGenerationScope::TextureView
                    | RenderGenerationScope::CommandEncoder
                    | RenderGenerationScope::CommandBuffer
                    | RenderGenerationScope::DependencyUtility => {
                        RenderGenerationScope::Objects.index()
                    }
                    _ => RenderGenerationScope::Root.index(),
                }
            }
        }
    }

    fn role_index(self, creation_position: usize) -> usize {
        match self {
            Self::Attempt => RenderPhase::ALL[creation_position].index(),
            Self::CpuInterval | Self::CpuSnapshot => RenderCpuScope::ALL[creation_position].index(),
            Self::GenerationInterval | Self::DeviceSnapshot => {
                RenderGenerationScope::ALL[creation_position].index()
            }
        }
    }
}

struct Fixture {
    domain: Domain,
    handles: Vec<Scope>,
    cpu_identity: RenderCpuIdentity,
    generation_identity: RenderGenerationIdentity,
    interval: IntervalReport<RenderWork, RenderResource>,
    profile: Report<RenderWork, RenderResource>,
    usage: RenderLifetimeUsage,
    foreign: Scope,
    foreign_refusal: Refusal<RenderWork, RenderResource>,
    foreign_event: Event<RenderWork, RenderResource>,
}

impl Fixture {
    fn new(domain: Domain) -> Self {
        NOW.set(0);
        let limits = Limits::new(
            RenderWorkTable {
                CoordinateVisits: 1,
                ..RenderWorkTable::splat(u64::MAX)
            },
            RenderResourceTable::splat(u64::MAX),
        );
        let mut operation = Operation::with_profile_clock(
            "unrelated root display label",
            limits,
            domain.scope_count(),
            ProfileMode::Detailed {
                timing: true,
                event_limit: 8,
                filter: surgeist_ledger::EventFilter::All,
            },
            clock,
        )
        .unwrap()
        .with_checkpoint_limit(1)
        .unwrap();
        let root = operation.root();
        // Creation order and semantic enum index are distinct. Match the
        // owner's semantic-indexed table, including its root-filled unused slots.
        let mut handles = vec![root; domain.scope_count()];
        for index in 1..domain.scope_count() {
            let child = operation
                .child(
                    handles[domain.parent(index)],
                    "unrelated child display label",
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .unwrap();
            handles[domain.role_index(index)] = child;
        }
        let checkpoint = operation.begin_checkpoint().unwrap();
        let origin = handles[1];
        NOW.set(2);
        operation
            .apply_batch(
                origin,
                &[
                    Change::Work(RenderWork::CoordinateVisits, 1),
                    Change::Acquire(RenderResource::ScratchCpuBytes, 3),
                    Change::Release(RenderResource::ScratchCpuBytes, 2),
                ],
            )
            .unwrap();
        NOW.set(4);
        assert!(matches!(
            operation.apply(origin, Change::Work(RenderWork::CoordinateVisits, 2)),
            Err(LedgerError::Refused(refusal))
                if refusal.scope == root && refusal.current == 1
                    && refusal.reason == (surgeist_ledger::RefusalReason::Limit { limit: 1 })
        ));
        NOW.set(5);
        operation.with_phase(origin, |_| NOW.set(9)).unwrap();
        let usage = RenderLifetimeUsage::new(*operation.usage(root).unwrap());
        let profile = operation.snapshot().unwrap().unwrap();
        let interval = operation.end_checkpoint(&checkpoint).unwrap();

        // A second real operation supplies an unknown identity, an actual
        // accepted event, and an actual limiting-scope refusal for corrupt inputs.
        let mut foreign_operation = Operation::with_profile(
            "foreign owner",
            Limits::new(
                RenderWorkTable {
                    CoordinateVisits: 0,
                    ..RenderWorkTable::splat(u64::MAX)
                },
                RenderResourceTable::splat(u64::MAX),
            ),
            1,
            ProfileMode::Detailed {
                timing: false,
                event_limit: 2,
                filter: surgeist_ledger::EventFilter::All,
            },
        )
        .unwrap();
        let foreign = foreign_operation.root();
        foreign_operation
            .apply(foreign, Change::Work(RenderWork::PaintVisits, 1))
            .unwrap();
        let LedgerError::Refused(foreign_refusal) = foreign_operation
            .apply(foreign, Change::Work(RenderWork::CoordinateVisits, 1))
            .unwrap_err()
        else {
            panic!("fixture must obtain an actual Ledger refusal");
        };
        let foreign_event = foreign_operation.snapshot().unwrap().unwrap().events[0];
        Self {
            domain,
            handles,
            cpu_identity: RenderCpuIdentity(root),
            generation_identity: RenderGenerationIdentity::new(),
            interval,
            profile,
            usage,
            foreign,
            foreign_refusal,
            foreign_event,
        }
    }

    fn edit_profiles(&mut self, mut edit: impl FnMut(&mut Report<RenderWork, RenderResource>)) {
        edit(&mut self.profile);
        edit(self.interval.profile.as_mut().unwrap());
    }

    fn corrupt(&mut self, corruption: Corruption) {
        let foreign = self.foreign;
        let foreign_refusal = self.foreign_refusal;
        let last = self.handles.len() - 1;
        match corruption {
            Corruption::DuplicateRow => {
                // The unused last role is replaced, keeping all event origins
                // known and both raw row counts unchanged.
                self.interval.scopes[last] = self.interval.scopes[0].clone();
                self.edit_profiles(|profile| profile.scopes[last] = profile.scopes[0].clone());
            }
            Corruption::RootOrder => {
                self.interval.scopes.swap(0, 1);
                self.edit_profiles(|profile| profile.scopes.swap(0, 1));
            }
            Corruption::ForeignRowAndEvent => {
                let foreign_event = self.foreign_event;
                self.interval.scopes[last].scope = foreign;
                self.edit_profiles(|profile| {
                    profile.scopes[last].scope = foreign;
                    profile.events.push(foreign_event);
                });
            }
            Corruption::Parent => {
                self.interval.scopes[1].parent = Some(foreign);
                self.edit_profiles(|profile| profile.scopes[1].parent = Some(foreign));
            }
            Corruption::SummaryRefusal => self.edit_profiles(|profile| {
                profile.scopes[1].first_refusal = Some(foreign_refusal);
            }),
            Corruption::RefusedLimit => self.edit_profiles(|profile| {
                let event = profile
                    .events
                    .iter_mut()
                    .find(|event| matches!(event, Event::Refused { .. }))
                    .unwrap();
                let Event::Refused { refusal, .. } = event else {
                    unreachable!()
                };
                *refusal = foreign_refusal;
            }),
            Corruption::AcceptedOrigin => self.edit_profiles(|profile| {
                let event = profile
                    .events
                    .iter_mut()
                    .find(|event| matches!(event, Event::Accepted { .. }))
                    .unwrap();
                let Event::Accepted { scope, .. } = event else {
                    unreachable!()
                };
                *scope = foreign;
            }),
            Corruption::PhaseOrigin => self.edit_profiles(|profile| {
                let event = profile
                    .events
                    .iter_mut()
                    .find(|event| matches!(event, Event::Phase { .. }))
                    .unwrap();
                let Event::Phase { scope, .. } = event else {
                    unreachable!()
                };
                *scope = foreign;
            }),
        }
    }

    fn build(self) -> std::result::Result<BuiltReport, RenderAccountingError> {
        match self.domain {
            Domain::Attempt => RenderAttemptReport::new(
                RenderAttemptKind::Frame,
                RenderAttemptStatus::Succeeded,
                self.interval,
                None,
            )
            .map(BuiltReport::Attempt),
            Domain::CpuInterval => RenderCpuInterval::new(self.cpu_identity, self.interval)
                .map(BuiltReport::CpuInterval),
            Domain::CpuSnapshot => {
                RenderCpuSnapshot::new(self.cpu_identity, self.usage, Some(self.profile), None)
                    .map(BuiltReport::CpuSnapshot)
            }
            Domain::GenerationInterval => {
                RenderGenerationInterval::new(self.generation_identity, self.interval)
                    .map(BuiltReport::GenerationInterval)
            }
            Domain::DeviceSnapshot => RenderDeviceSnapshot::new(
                self.generation_identity,
                self.usage,
                Some(self.profile),
                false,
                None,
            )
            .map(BuiltReport::DeviceSnapshot),
        }
    }
}

#[derive(Clone, Copy)]
enum Corruption {
    DuplicateRow,
    RootOrder,
    ForeignRowAndEvent,
    Parent,
    SummaryRefusal,
    RefusedLimit,
    AcceptedOrigin,
    PhaseOrigin,
}

#[derive(Debug)]
enum BuiltReport {
    Attempt(RenderAttemptReport),
    CpuInterval(RenderCpuInterval),
    CpuSnapshot(RenderCpuSnapshot),
    GenerationInterval(RenderGenerationInterval),
    DeviceSnapshot(RenderDeviceSnapshot),
}

fn assert_rejected(domain: Domain, corruption: Corruption) {
    let mut fixture = Fixture::new(domain);
    fixture.corrupt(corruption);
    assert_invariant(fixture.build());
}

fn assert_invariant(result: std::result::Result<BuiltReport, RenderAccountingError>) {
    match result {
        Err(error) => assert_eq!(error, RenderAccountingError::Invariant),
        Ok(_) => panic!("report construction accepted an invalid scope mapping"),
    }
}

// Literal expectations come from the admitted batch, actual refusal, and ended
// phase above. They do not project or derive expected values from the raw report.
macro_rules! expected_events {
    ($event:ident, $field:ident, $scope:path) => {
        [
            $event::Change {
                $field: $scope,
                quantity: RenderQuantity::Work(RenderWork::CoordinateVisits),
                amount: 1,
                kind: RenderChangeKind::Work,
                elapsed: Some(Duration::from_millis(2)),
            },
            $event::Change {
                $field: $scope,
                quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                amount: 3,
                kind: RenderChangeKind::Acquire,
                elapsed: Some(Duration::from_millis(2)),
            },
            $event::Change {
                $field: $scope,
                quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                amount: 2,
                kind: RenderChangeKind::Release,
                elapsed: Some(Duration::from_millis(2)),
            },
            $event::Refused {
                $field: $scope,
                elapsed: Some(Duration::from_millis(4)),
            },
            $event::Phase {
                $field: $scope,
                duration: Some(Duration::from_millis(4)),
                elapsed: Some(Duration::from_millis(9)),
            },
        ]
    };
}

fn assert_common_metadata(
    elapsed: Option<Duration>,
    count: u64,
    duration: Option<Duration>,
    dropped: u64,
    saturated: bool,
) {
    assert_eq!(elapsed, Some(Duration::from_millis(9)));
    assert_eq!(count, 1);
    assert_eq!(duration, Some(Duration::from_millis(4)));
    assert_eq!(dropped, 0);
    assert!(!saturated);
}

fn assert_valid_observation(domain: Domain) {
    match Fixture::new(domain).build().unwrap() {
        BuiltReport::Attempt(report) => {
            assert_eq!(
                report.events().collect::<Vec<_>>(),
                expected_events!(RenderEvent, phase, RenderPhase::Preparation)
            );
            let usage = report.phase_usage(RenderPhase::Preparation);
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 1);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 1);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 3);
            assert_common_metadata(
                report.elapsed(),
                report.phase_count(RenderPhase::Preparation),
                report.phase_elapsed(RenderPhase::Preparation),
                report.dropped_events(),
                report.saturated(),
            );
        }
        BuiltReport::CpuInterval(report) => {
            assert_eq!(
                report.events().collect::<Vec<_>>(),
                expected_events!(RenderCpuEvent, scope, RenderCpuScope::FrameBacking)
            );
            let usage = report.scope_usage(RenderCpuScope::FrameBacking);
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 1);
            assert_eq!(usage.opening_live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 1);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 3);
            assert_common_metadata(
                report.elapsed(),
                report.scope_count(RenderCpuScope::FrameBacking),
                report.scope_elapsed(RenderCpuScope::FrameBacking),
                report.dropped_events(),
                report.saturated(),
            );
        }
        BuiltReport::CpuSnapshot(report) => {
            assert_eq!(
                report.events().collect::<Vec<_>>(),
                expected_events!(RenderCpuEvent, scope, RenderCpuScope::FrameBacking)
            );
            let usage = report.scope_usage(RenderCpuScope::FrameBacking).unwrap();
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 1);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 1);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 3);
            assert_common_metadata(
                report.elapsed(),
                report.scope_count(RenderCpuScope::FrameBacking),
                report.scope_elapsed(RenderCpuScope::FrameBacking),
                report.dropped_events(),
                report.saturated(),
            );
        }
        BuiltReport::GenerationInterval(report) => {
            assert_eq!(
                report.events().collect::<Vec<_>>(),
                expected_events!(RenderGenerationEvent, scope, RenderGenerationScope::Cache)
            );
            let usage = report.scope_usage(RenderGenerationScope::Cache);
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 1);
            assert_eq!(
                report.scope_opening(
                    RenderGenerationScope::Cache,
                    RenderResource::ScratchCpuBytes
                ),
                0
            );
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 1);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 3);
            assert_common_metadata(
                report.elapsed(),
                report.scope_count(RenderGenerationScope::Cache),
                report.scope_elapsed(RenderGenerationScope::Cache),
                report.dropped_events(),
                report.saturated(),
            );
        }
        BuiltReport::DeviceSnapshot(report) => {
            assert_eq!(
                report.events().collect::<Vec<_>>(),
                expected_events!(RenderGenerationEvent, scope, RenderGenerationScope::Cache)
            );
            let usage = report.scope_usage(RenderGenerationScope::Cache).unwrap();
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 1);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 1);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 3);
            assert_common_metadata(
                report.elapsed(),
                report.scope_count(RenderGenerationScope::Cache),
                report.scope_elapsed(RenderGenerationScope::Cache),
                report.dropped_events(),
                report.saturated(),
            );
        }
    }
}

macro_rules! domain_tests {
    ($module:ident, $domain:ident) => {
        mod $module {
            use super::*;
            #[test]
            fn duplicate_rows_with_unchanged_count_are_rejected() {
                assert_rejected(Domain::$domain, Corruption::DuplicateRow);
            }
            #[test]
            fn a_child_cannot_replace_the_registered_root() {
                assert_rejected(Domain::$domain, Corruption::RootOrder);
            }
            #[test]
            fn a_foreign_event_is_not_authorized_by_an_equally_sized_raw_table() {
                assert_rejected(Domain::$domain, Corruption::ForeignRowAndEvent);
            }
            #[test]
            fn an_unknown_parent_is_rejected() {
                assert_rejected(Domain::$domain, Corruption::Parent);
            }
            #[test]
            fn an_unknown_summary_refusal_scope_is_rejected() {
                assert_rejected(Domain::$domain, Corruption::SummaryRefusal);
            }
            #[test]
            fn an_unknown_limiting_scope_is_rejected_even_with_a_known_origin() {
                assert_rejected(Domain::$domain, Corruption::RefusedLimit);
            }
            #[test]
            fn unknown_accepted_and_phase_origins_are_rejected() {
                // Characterization: the existing constructors already reject
                // these origins when the raw rows do not claim to authorize them.
                assert_rejected(Domain::$domain, Corruption::AcceptedOrigin);
                assert_rejected(Domain::$domain, Corruption::PhaseOrigin);
            }
            #[test]
            fn valid_events_keep_literal_values_order_and_timing_despite_labels() {
                assert_valid_observation(Domain::$domain);
            }
        }
    };
}

domain_tests!(attempt, Attempt);
domain_tests!(cpu_interval, CpuInterval);
domain_tests!(cpu_snapshot, CpuSnapshot);
domain_tests!(generation_interval, GenerationInterval);
domain_tests!(device_snapshot, DeviceSnapshot);

#[test]
fn cpu_interval_rejects_an_identity_from_another_operation() {
    let mut fixture = Fixture::new(Domain::CpuInterval);
    fixture.cpu_identity = RenderCpuIdentity(fixture.foreign);
    assert_invariant(fixture.build());
}

#[test]
fn cpu_snapshot_rejects_an_identity_from_another_operation() {
    let mut fixture = Fixture::new(Domain::CpuSnapshot);
    fixture.cpu_identity = RenderCpuIdentity(fixture.foreign);
    assert_invariant(fixture.build());
}

#[test]
fn owner_kernel_work_is_read_back_under_its_semantic_phase() {
    // This uses Render's actual producer and unmodified Ledger output. The
    // declared phase owns the work regardless of physical scope creation order.
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Kernel, |account| {
            account.work(RenderWork::CoordinateVisits, 1)
        })
        .unwrap();
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(
        report
            .phase_usage(RenderPhase::Kernel)
            .work(RenderWork::CoordinateVisits),
        1
    );
}

#[test]
fn owner_kernel_work_reaches_its_declared_gpu_and_render_parents() {
    let mut context = RenderAccountingContext::default();
    let parents = context
        .run(RenderAttemptKind::Frame, RenderPhase::Kernel, |account| {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let work = |phase: RenderPhase| {
                account
                    .ledger
                    .usage(account.scopes[phase.index()])
                    .unwrap()
                    .work(RenderWork::CoordinateVisits)
            };
            assert_eq!(work(RenderPhase::Kernel), 1);
            // Read Ledger directly, independently of Render's report-row projection.
            Ok((work(RenderPhase::GpuPrepare), work(RenderPhase::Render)))
        })
        .unwrap();
    assert_eq!(parents, (1, 1));
}

#[test]
fn owner_acquire_work_reaches_its_declared_present_and_frame_parents() {
    let mut context = RenderAccountingContext::default();
    let parents = context
        .run(RenderAttemptKind::Frame, RenderPhase::Acquire, |account| {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let work = |phase: RenderPhase| {
                account
                    .ledger
                    .usage(account.scopes[phase.index()])
                    .unwrap()
                    .work(RenderWork::CoordinateVisits)
            };
            assert_eq!(work(RenderPhase::Acquire), 1);
            Ok((work(RenderPhase::Present), work(RenderPhase::Frame)))
        })
        .unwrap();
    assert_eq!(parents, (1, 1));
}
