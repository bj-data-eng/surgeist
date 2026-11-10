use super::*;
use crate::accounting::{
    RenderAccountingOptions, RenderClock, RenderCpuSnapshot, RenderDeviceSnapshot,
    RenderGenerationScope, RenderProfile, RendererCpu,
};
use crate::resource::{ResourceManager, ResourceManagerObservationForTest};
use crate::{ErrorCode, ResourceCacheBudget};
use std::time::Duration;
use surgeist_ledger::Dimension;

fn profiles() -> [RenderProfile; 3] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ]
}

#[derive(Debug, Eq, PartialEq)]
struct ScratchBacking {
    frame: FrameIdentity,
    indices: (*const (u64, ResourceIdentity), usize),
    evicted: (*const ResourceIdentity, usize),
}

struct Observation {
    cpu: RenderCpuSnapshot,
    generation: RenderDeviceSnapshot,
    registry: ResourceManagerObservationForTest,
    audit_probes: u64,
    scratch: Vec<ScratchBacking>,
}

#[derive(Default)]
struct ProofWork {
    entry_audits: u64,
    cache_probes: u64,
    cpu_probes: u64,
    lease_returns: u64,
}

struct Fixture {
    profile: RenderProfile,
    cpu: RendererCpu,
    manager: ResourceManager,
}

impl Fixture {
    fn new(profile: RenderProfile) -> Self {
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(|| Duration::ZERO));
        let cpu = RendererCpu::new(options).unwrap();
        let manager = ResourceManager::with_generation(
            ResourceCacheBudget::DISABLED,
            Generation::new(options).unwrap(),
            cpu.at(RenderCpuScope::DeviceBacking),
        );
        Self {
            profile,
            cpu,
            manager,
        }
    }

    fn frame(&self) -> FrameResourceScope {
        self.manager.begin_frame(None).unwrap()
    }

    fn observe(&self) -> Observation {
        let generation = self.manager.generation();
        let scratch = lock_state(&self.manager.state)
            .active_frames
            .iter()
            .map(|(&frame, scratch)| ScratchBacking {
                frame,
                indices: (scratch.indices.as_ptr(), scratch.indices.capacity()),
                evicted: (scratch.evicted.as_ptr(), scratch.evicted.capacity()),
            })
            .collect();
        Observation {
            cpu: self.cpu.snapshot().unwrap(),
            generation: generation.snapshot().unwrap(),
            registry: self.manager.observation_for_test(),
            audit_probes: generation
                .class_usage(GenerationClass::CacheVello)
                .unwrap()
                .work(RenderWork::ResourceProbes),
            scratch,
        }
    }

    fn check(&self, expected: ProofWork, prove: impl FnOnce() -> Result<()>) -> Result<()> {
        let before = self.observe();
        let result = prove();
        let after = self.observe();
        assert_eq!(
            after.audit_probes - before.audit_probes,
            expected.entry_audits
        );
        let cpu_before = before.cpu.usage();
        let cpu_after = after.cpu.usage();
        let generation_before = before.generation.usage();
        let generation_after = after.generation.usage();
        for &work in RenderWork::ALL {
            assert_eq!(
                cpu_after.work(work) - cpu_before.work(work),
                if work == RenderWork::ResourceProbes {
                    expected.cpu_probes
                } else {
                    0
                },
                "CPU {work:?}"
            );
            assert_eq!(
                generation_after.work(work) - generation_before.work(work),
                match work {
                    RenderWork::ResourceProbes => expected.entry_audits + expected.cache_probes,
                    RenderWork::ResourceLeaseReturns => expected.lease_returns,
                    _ => 0,
                },
                "Generation {work:?}"
            );
        }
        for &resource in RenderResource::ALL {
            assert_eq!(cpu_after.live(resource), cpu_before.live(resource));
            assert_eq!(cpu_after.peak(resource), cpu_before.peak(resource));
            assert_eq!(
                generation_after.live(resource),
                generation_before.live(resource)
            );
            assert_eq!(
                generation_after.peak(resource),
                generation_before.peak(resource)
            );
        }
        assert_eq!(after.scratch, before.scratch);
        assert_eq!(
            after.registry.active_frame_count,
            before.registry.active_frame_count
        );
        assert_eq!(
            after.registry.entry_identities_for_test(),
            before.registry.entry_identities_for_test()
        );
        assert_eq!(after.registry.leased_count, before.registry.leased_count);
        assert_eq!(after.registry.idle_count, before.registry.idle_count);
        assert_eq!(
            after.registry.resolved_lease_count,
            before.registry.resolved_lease_count + expected.lease_returns as usize
        );
        assert_eq!(after.registry.accounting_fault_for_test(), None);
        assert_eq!(after.cpu.fault(), None);
        assert_eq!(after.generation.fault(), None);
        assert_eq!(
            after.cpu.scope_count(RenderCpuScope::Cleanup)
                - before.cpu.scope_count(RenderCpuScope::Cleanup),
            u64::from(!matches!(self.profile, RenderProfile::Off))
        );
        assert_eq!(
            after.cpu.scope_elapsed(RenderCpuScope::Cleanup),
            matches!(self.profile, RenderProfile::Summary { timing: true })
                .then_some(Duration::ZERO)
        );
        for scope in RenderGenerationScope::ALL {
            assert_eq!(
                after.generation.scope_count(scope),
                before.generation.scope_count(scope)
            );
            assert_eq!(
                after.generation.scope_elapsed(scope),
                before.generation.scope_elapsed(scope)
            );
        }
        result
    }

    fn qualify(&self, identity: &ManagerIdentity, frame: FrameIdentity) -> Result<()> {
        let mut state = lock_state(&self.manager.state);
        let cpu = state.cpu.at(RenderCpuScope::Cleanup);
        cleanup_phase(&cpu, || {
            state.ensure_frame_resolution_ready(identity, frame, std::iter::empty())
        })
    }

    fn assert_closed(&self) {
        let after = self.observe();
        assert_eq!(after.registry.active_frame_count, 0);
        assert_eq!(after.registry.entry_count, 0);
        assert_eq!(after.registry.resolved_lease_count, 0);
        assert_eq!(after.registry.accounted_entry_bytes, Some(0));
        assert_eq!(after.registry.accounting_fault_for_test(), None);
        assert_eq!(after.cpu.usage().live(RenderResource::ScratchCpuBytes), 0);
        for resource in [
            RenderResource::GpuBytes,
            RenderResource::RegisteredCacheGpuBytes,
            RenderResource::IdleCacheGpuBytes,
        ] {
            assert_eq!(after.generation.usage().live(resource), 0);
        }
    }
}

fn acquire(frame: &mut FrameResourceScope, bytes: u64) -> ResourceLease {
    frame.acquire(ResourceCacheKey::VelloAtlas, bytes).unwrap()
}

fn assert_lease_error(error: Error, lease: &ResourceLease, invariant: &str) {
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    let value = error.invalid_value_diagnostic().unwrap();
    assert_eq!(value.field(), "resource lease");
    assert_eq!(value.value(), lease.resource_identity().get().to_string());
    assert_eq!(value.invariant(), invariant);
}

fn assert_render_failure(error: Error, message: &str) {
    assert_eq!(error.code(), ErrorCode::RenderFailed);
    assert_eq!(error.message(), message);
}

#[test]
fn frame_and_token_refusals_preserve_order_and_allow_retry() {
    for profile in profiles() {
        let fixture = Fixture::new(profile);
        let former = fixture.frame();
        let former_frame = former.frame;
        drop(former.finish());
        let mut other = fixture.frame();
        let wrong_frame = acquire(&mut other, 4);
        let mut receiving = fixture.frame();
        let own = acquire(&mut receiving, 8);
        let foreign_fixture = Fixture::new(profile);
        let mut foreign_frame = foreign_fixture.frame();
        let foreign = acquire(&mut foreign_frame, 4);
        assert_ne!(foreign_frame.frame, receiving.frame);

        // The initial accounting audit visits the two receiving-manager entries.
        for (identity, frame) in [
            (&receiving.manager_identity, former_frame),
            (&foreign_frame.manager_identity, receiving.frame),
        ] {
            let error = fixture
                .check(
                    ProofWork {
                        entry_audits: 2,
                        ..ProofWork::default()
                    },
                    || fixture.qualify(identity, frame),
                )
                .unwrap_err();
            assert_render_failure(
                error,
                "resource frame is unavailable for clean accounting resolution",
            );
        }

        for (tokens, invariant) in [
            (
                [&foreign, &wrong_frame],
                "must belong to this resource manager",
            ),
            (
                [&wrong_frame, &foreign],
                "must belong to this active resource frame",
            ),
        ] {
            // Two seed visits precede the first identity check. Entry validation
            // has no token to probe after this refusal.
            let error = fixture
                .check(
                    ProofWork {
                        entry_audits: 2,
                        cpu_probes: 2,
                        ..ProofWork::default()
                    },
                    || receiving.ensure_commit_ready(&tokens),
                )
                .unwrap_err();
            assert_lease_error(error, tokens[0], invariant);
        }

        // Two seed visits, one push and two coverage visits.
        fixture
            .check(
                ProofWork {
                    entry_audits: 2,
                    cache_probes: 1,
                    cpu_probes: 5,
                    ..ProofWork::default()
                },
                || receiving.ensure_commit_ready(&[&own]),
            )
            .unwrap();
        receiving.release(own).unwrap();
        drop(receiving.finish_checked().unwrap());
        other.release(wrong_frame).unwrap();
        drop(other.finish_checked().unwrap());
        foreign_frame.release(foreign).unwrap();
        drop(foreign_frame.finish_checked().unwrap());
        fixture.assert_closed();
        foreign_fixture.assert_closed();
    }
}

#[test]
fn complete_proof_checks_coverage_and_seeds_resolved_leases() {
    for profile in profiles() {
        let fixture = Fixture::new(profile);
        let mut frame = fixture.frame();
        let first = acquire(&mut frame, 4);
        let second = acquire(&mut frame, 8);
        assert!(first.resource_identity() < second.resource_identity());

        // Coverage refuses at the first registry entry in both cases.
        for (tokens, pushes) in [(&[][..], 0), (&[&second][..], 1)] {
            let error = fixture
                .check(
                    ProofWork {
                        entry_audits: 2,
                        cache_probes: pushes,
                        cpu_probes: 2 + pushes + 1,
                        ..ProofWork::default()
                    },
                    || frame.ensure_commit_ready(tokens),
                )
                .unwrap_err();
            assert_render_failure(
                error,
                "clean resource resolution must own every exact lease in its frame",
            );
        }

        // The actual atomic operation resolves the first entry and leaves its
        // borrowed lease available to exercise the resolved-token check.
        fixture
            .check(
                ProofWork {
                    cache_probes: 2,
                    cpu_probes: 1,
                    lease_returns: 1,
                    ..ProofWork::default()
                },
                || frame.resolve_leases_atomically(&[&first]),
            )
            .unwrap();
        let error = fixture
            .check(
                ProofWork {
                    entry_audits: 2,
                    cache_probes: 1,
                    cpu_probes: 3,
                    ..ProofWork::default()
                },
                || frame.ensure_commit_ready(&[&first, &second]),
            )
            .unwrap_err();
        assert_lease_error(
            error,
            &first,
            "must not have already been resolved by this frame",
        );

        // Two seed visits, two pushes, one sort and one pair comparison,
        // then two coverage visits account for the mixed proof.
        fixture
            .check(
                ProofWork {
                    entry_audits: 2,
                    cache_probes: 1,
                    cpu_probes: 8,
                    ..ProofWork::default()
                },
                || frame.ensure_commit_ready(&[&second]),
            )
            .unwrap();
        drop(first);
        frame.release(second).unwrap();
        let cleanup = frame.finish_checked().unwrap();
        assert_eq!(cleanup.evicted_resources().len(), 2);
        drop(cleanup);
        fixture.assert_closed();
    }
}

#[test]
fn atomic_proof_refuses_duplicates_and_capacity_before_replay() {
    for profile in profiles() {
        let fixture = Fixture::new(profile);
        let mut frame = fixture.frame();
        let first = acquire(&mut frame, 4);
        let second = acquire(&mut frame, 8);
        let capacity = lock_state(&frame.state)
            .active_frames
            .get(&frame.frame)
            .unwrap()
            .indices
            .capacity();
        assert!(capacity >= 2);

        // Two validation probes, two pushes, a sort and an adjacent comparison.
        let error = fixture
            .check(
                ProofWork {
                    cache_probes: 2,
                    cpu_probes: 4,
                    ..ProofWork::default()
                },
                || frame.resolve_leases_atomically(&[&second, &second]),
            )
            .unwrap_err();
        assert_lease_error(
            error,
            &second,
            "must occur at most once in one clean resolution",
        );

        // The extra real token reaches validation before the capacity refusal.
        let repeated = vec![&second; capacity + 1];
        let error = fixture
            .check(
                ProofWork {
                    cache_probes: capacity as u64 + 1,
                    cpu_probes: capacity as u64,
                    ..ProofWork::default()
                },
                || frame.resolve_leases_atomically(&repeated),
            )
            .unwrap_err();
        assert_lease_error(
            error,
            &second,
            "must occur at most once in the active frame",
        );

        // Both prior refusals leave the leases available for the real replay.
        fixture
            .check(
                ProofWork {
                    cache_probes: 4,
                    cpu_probes: 4,
                    lease_returns: 2,
                    ..ProofWork::default()
                },
                || frame.resolve_leases_atomically(&[&second, &first]),
            )
            .unwrap();
        drop((first, second));
        let cleanup = frame.finish_checked().unwrap();
        assert_eq!(cleanup.evicted_resources().len(), 2);
        drop(cleanup);
        fixture.assert_closed();
    }
}
