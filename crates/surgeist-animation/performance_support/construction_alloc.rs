//! Construction and retained-storage measurements for the separate allocator executable.

use std::hint::black_box;
use std::mem::{size_of, size_of_val};

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats};
use surgeist_animation::{
    AnimationPlayState, ElapsedTime, InterpolableValue, InterpolationError, InterpolationPair,
    KeyframeSegment, KeyframeTrack, NonFiniteInterpolationResult, SampledPropertyResult,
    TransitionTrack,
};

use super::fixtures::{Family, Layout};
use super::lifecycle::{self, Curve, KeyReuse};

pub fn report_construction(filter: &str, iterations: usize, check_single: bool) -> usize {
    let mut matched = 0;
    let mut checked = 0;
    let mut failures = 0;
    for layout in Layout::ALL {
        for properties in [1, 2, 8, 64] {
            for segments in [1, 2, 8, 64] {
                let name = format!(
                    "construct/track_only/{}/p{properties}/s{segments}",
                    layout.name()
                );
                if !name.contains(filter) {
                    continue;
                }
                matched += 1;
                let prepared = lifecycle::segments(
                    properties,
                    segments,
                    layout,
                    Family::Number,
                    KeyReuse::PerSegment,
                    Curve::Linear,
                );
                let mut total = Stats::default();
                for _ in 0..iterations {
                    // Every caller input is prepared before the measured constructor.
                    let input = prepared.clone();
                    let id = lifecycle::animation_id();
                    let timing = lifecycle::timing();
                    let region = Region::new(&INSTRUMENTED_SYSTEM);
                    let track = KeyframeTrack::new(id, timing, input).unwrap();
                    let stats = region.change();
                    black_box(&track);
                    drop(track);
                    accumulate(&mut total, stats);
                }
                println!(
                    "{name},{iterations},{},{},{},{},{}",
                    total.allocations,
                    total.reallocations,
                    total.deallocations,
                    total.bytes_allocated,
                    total.bytes_reallocated
                );
                if check_single && properties == 1 {
                    checked += 1;
                    if total.allocations != 0 || total.reallocations != 0 {
                        failures += 1;
                        eprintln!(
                            "{name}: expected zero constructor allocations and reallocations, got {total:?}"
                        );
                    }
                }
            }
        }
    }
    if check_single {
        assert!(checked > 0, "filter matched no constructor contracts");
        assert_eq!(
            failures, 0,
            "single-property constructor allocation contracts failed"
        );
    }
    matched
}

fn accumulate(total: &mut Stats, value: Stats) {
    total.allocations += value.allocations;
    total.reallocations += value.reallocations;
    total.deallocations += value.deallocations;
    total.bytes_allocated += value.bytes_allocated;
    total.bytes_deallocated += value.bytes_deallocated;
    total.bytes_reallocated += value.bytes_reallocated;
}

fn retained<T>(filter: &str, name: &str, construct: impl FnOnce() -> T) -> usize {
    if !name.contains(filter) {
        return 0;
    }
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let owner = construct();
    let stats = region.change();
    let inline = size_of_val(&owner);
    // stats_alloc already incorporates realloc growth/shrink in these totals.
    let heap = stats
        .bytes_allocated
        .checked_sub(stats.bytes_deallocated)
        .expect("complete construction must not free pre-existing allocations");
    black_box(&owner);
    drop(owner);
    println!(
        "{name},{heap},{inline},{},{},{}",
        heap + inline,
        stats.allocations,
        stats.reallocations
    );
    1
}

pub fn report_memory(filter: &str) {
    println!("case,heap_bytes,inline_bytes,total_bytes,allocations,reallocations");
    let mut matched = 0;
    for (name, bytes) in [
        ("InterpolationPair", size_of::<InterpolationPair>()),
        ("KeyframeSegment", size_of::<KeyframeSegment>()),
        ("InterpolableValue", size_of::<InterpolableValue>()),
        ("InterpolationError", size_of::<InterpolationError>()),
        (
            "NonFiniteInterpolationResult",
            size_of::<NonFiniteInterpolationResult>(),
        ),
        ("SampledPropertyResult", size_of::<SampledPropertyResult>()),
        ("TransitionTrack", size_of::<TransitionTrack>()),
        ("KeyframeTrack", size_of::<KeyframeTrack>()),
    ] {
        let name = format!("type/{name}");
        if name.contains(filter) {
            matched += 1;
            println!("{name},0,{bytes},{bytes},0,0");
        }
    }
    let midpoint = ElapsedTime::from_secs(0.1).unwrap();
    for family in Family::ALL {
        let prefix = format!("memory/transition/{}", family.name());
        matched += retained(filter, &format!("{prefix}/track"), || {
            lifecycle::transition(family, Curve::Linear)
        });
        matched += retained(filter, &format!("{prefix}/with_result"), || {
            let track = lifecycle::transition(family, Curve::Linear);
            let sample = track.sample(midpoint, AnimationPlayState::Running);
            (track, sample)
        });
        matched += retained(filter, &format!("{prefix}/with_clone"), || {
            let track = lifecycle::transition(family, Curve::Linear);
            let cloned = track.clone();
            (track, cloned)
        });
        matched += retained(filter, &format!("{prefix}/result_after_drop"), || {
            lifecycle::transition(family, Curve::Linear)
                .sample(midpoint, AnimationPlayState::Running)
        });
        for layout in Layout::ALL {
            for properties in lifecycle::PROPERTY_COUNTS {
                for segments in lifecycle::SEGMENT_COUNTS {
                    for reuse in KeyReuse::ALL {
                        let prefix = format!(
                            "memory/keyframe/{}/{}/p{properties}/s{segments}/{}",
                            family.name(),
                            layout.name(),
                            reuse.name()
                        );
                        let construct = || {
                            lifecycle::track(lifecycle::segments(
                                properties,
                                segments,
                                layout,
                                family,
                                reuse,
                                Curve::Linear,
                            ))
                        };
                        matched += retained(filter, &format!("{prefix}/track"), construct);
                        matched += retained(filter, &format!("{prefix}/with_result"), || {
                            let track = construct();
                            let samples = track.sample(midpoint, AnimationPlayState::Running);
                            (track, samples)
                        });
                        matched += retained(filter, &format!("{prefix}/with_clone"), || {
                            let track = construct();
                            let cloned = track.clone();
                            (track, cloned)
                        });
                        matched += retained(filter, &format!("{prefix}/result_after_drop"), || {
                            construct().sample(midpoint, AnimationPlayState::Running)
                        });
                    }
                }
            }
        }
    }
    assert!(matched > 0, "filter matched no memory measurement cases");
}
