//! Run separately from throughput timing: this global allocator instruments all
//! allocations. Fixtures, argument parsing, and CSV reporting are outside regions.

#![forbid(unsafe_code)]

#[path = "../performance_support/fixtures.rs"]
mod fixtures;

#[path = "../performance_support/lifecycle.rs"]
mod lifecycle;

#[path = "../performance_support/construction_alloc.rs"]
mod construction_alloc;

#[path = "../performance_support/intake_alloc.rs"]
mod intake_alloc;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;
use std::cell::Cell;
use std::hint::black_box;
use surgeist_animation::AnimationPlayState;

use fixtures::{Family, Layout, SIZES};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

struct Config {
    filter: String,
    iterations: usize,
    matched: Cell<usize>,
    check_values: bool,
    check_failures: bool,
    check_reuse: bool,
    check_construction: bool,
    check_intake: bool,
    report_memory: bool,
    checked: Cell<usize>,
    failures: Cell<usize>,
}

impl Config {
    fn read() -> Self {
        let mut result = Self {
            filter: String::new(),
            iterations: 100,
            matched: Cell::new(0),
            check_values: false,
            check_failures: false,
            check_reuse: false,
            check_construction: false,
            check_intake: false,
            report_memory: false,
            checked: Cell::new(0),
            failures: Cell::new(0),
        };
        let mut explicit_mode_seen = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            if matches!(
                arg.as_str(),
                "--report-only"
                    | "--check-values"
                    | "--check-failures"
                    | "--check-reuse"
                    | "--check-construction"
                    | "--check-intake"
                    | "--report-memory"
            ) {
                assert!(
                    !explicit_mode_seen,
                    "allocation-check modes are mutually exclusive and cannot be repeated"
                );
                explicit_mode_seen = true;
            }
            match arg.as_str() {
                "--report-only" => {}
                "--report-memory" => result.report_memory = true,
                "--check-intake" => result.check_intake = true,
                "--check-construction" => {
                    result.check_construction = true;
                    result.filter = "construct/track_only/".to_owned();
                }
                "--check-values" => {
                    result.check_values = true;
                    result.filter = "transition/sample/".to_owned();
                }
                "--check-failures" => {
                    result.check_failures = true;
                    result.filter = "transition/sample/arithmetic_failure".to_owned();
                }
                "--check-reuse" => {
                    result.check_reuse = true;
                    result.filter = "keyframe/sample_into/".to_owned();
                }
                "--filter" => result.filter = args.next().expect("--filter needs text"),
                "--iterations" => {
                    result.iterations = args
                        .next()
                        .expect("--iterations needs a value")
                        .parse()
                        .expect("--iterations needs an integer");
                    assert!(result.iterations > 0, "iterations must be positive");
                }
                "--help" => {
                    println!(
                        "allocation_check [--report-only | --check-values | --check-failures | --check-reuse | --check-construction | --check-intake | --report-memory] [--filter SUBSTRING] [--iterations 100]"
                    );
                    std::process::exit(0);
                }
                _ => panic!("unknown argument: {arg}"),
            }
        }
        result
    }
}

fn report(config: &Config, name: &str, mut run: impl FnMut(usize)) {
    if !name.contains(&config.filter) {
        return;
    }
    config.matched.set(config.matched.get() + 1);
    run(0);
    let region = Region::new(GLOBAL);
    for index in 0..config.iterations {
        run(index);
    }
    let stats = region.change();
    if (config.check_values
        && name.starts_with("transition/sample/")
        && !name.ends_with("arithmetic_failure"))
        || (config.check_failures && name == "transition/sample/arithmetic_failure")
        || (config.check_reuse && name.starts_with("keyframe/sample_into/"))
    {
        config.checked.set(config.checked.get() + 1);
        if stats.allocations != 0 || stats.reallocations != 0 {
            config.failures.set(config.failures.get() + 1);
            eprintln!("{name}: expected zero allocations and reallocations, got {stats:?}");
        }
    }
    println!(
        "{name},{},{},{},{},{},{}",
        config.iterations,
        stats.allocations,
        stats.reallocations,
        stats.deallocations,
        stats.bytes_allocated,
        stats.bytes_reallocated
    );
}

fn main() {
    let config = Config::read();
    if config.check_intake {
        intake_alloc::report(&config.filter, config.iterations);
        return;
    }
    if config.report_memory {
        construction_alloc::report_memory(&config.filter);
        return;
    }
    println!(
        "case,iterations,allocations,reallocations,deallocations,bytes_allocated,bytes_reallocated"
    );
    let elapsed = fixtures::elapsed_inputs();
    for family in Family::ALL {
        report(
            &config,
            &format!("transition/construct/{}", family.name()),
            |_| {
                black_box(fixtures::transition(black_box(family)));
            },
        );
        let transition = fixtures::transition(family);
        report(
            &config,
            &format!("transition/sample/{}", family.name()),
            |index| {
                black_box(transition.sample(
                    black_box(elapsed[index % elapsed.len()]),
                    AnimationPlayState::Running,
                ));
            },
        );
        let track = fixtures::track(fixtures::segments(1, 1, Layout::Grouped, family));
        report(
            &config,
            &format!("keyframe/sample/family/{}", family.name()),
            |index| {
                black_box(track.sample(
                    black_box(elapsed[index % elapsed.len()]),
                    AnimationPlayState::Running,
                ));
            },
        );
        let mut output = Vec::with_capacity(1);
        report(
            &config,
            &format!("keyframe/sample_into/family/{}", family.name()),
            |index| {
                track.sample_into(
                    black_box(elapsed[index % elapsed.len()]),
                    AnimationPlayState::Running,
                    &mut output,
                );
                black_box(output.as_slice());
            },
        );
    }
    for layout in Layout::ALL {
        for properties in SIZES {
            for per_property in SIZES {
                let suffix = format!("{}/p{properties}/s{per_property}", layout.name());
                report(&config, &format!("construct/complete/{suffix}"), |_| {
                    black_box(fixtures::track(fixtures::segments(
                        black_box(properties),
                        black_box(per_property),
                        layout,
                        Family::Number,
                    )));
                });
                let prepared = fixtures::segments(properties, per_property, layout, Family::Number);
                let name = format!("construct/prepared/{suffix}");
                if name.contains(&config.filter) {
                    config.matched.set(config.matched.get() + 1);
                    // Each input clone is outside its own measured region. Report
                    // the sum; no caller-input allocations enter this boundary.
                    let mut total = stats_alloc::Stats::default();
                    for _ in 0..config.iterations {
                        let input = prepared.clone();
                        let region = Region::new(GLOBAL);
                        black_box(fixtures::track(black_box(input)));
                        let stats = region.change();
                        total.allocations += stats.allocations;
                        total.reallocations += stats.reallocations;
                        total.deallocations += stats.deallocations;
                        total.bytes_allocated += stats.bytes_allocated;
                        total.bytes_reallocated += stats.bytes_reallocated;
                    }
                    println!(
                        "{name},{},{},{},{},{},{}",
                        config.iterations,
                        total.allocations,
                        total.reallocations,
                        total.deallocations,
                        total.bytes_allocated,
                        total.bytes_reallocated
                    );
                }
                let track = fixtures::track(prepared);
                report(&config, &format!("keyframe/sample/{suffix}"), |index| {
                    black_box(track.sample(
                        black_box(elapsed[index % elapsed.len()]),
                        AnimationPlayState::Running,
                    ));
                });
                let mut output = Vec::with_capacity(properties);
                report(
                    &config,
                    &format!("keyframe/sample_into/{suffix}"),
                    |index| {
                        track.sample_into(
                            black_box(elapsed[index % elapsed.len()]),
                            AnimationPlayState::Running,
                            &mut output,
                        );
                        black_box(output.as_slice());
                    },
                );
            }
        }
    }
    config.matched.set(
        config.matched.get()
            + construction_alloc::report_construction(
                &config.filter,
                config.iterations,
                config.check_construction,
            ),
    );
    assert!(
        config.matched.get() > 0,
        "filter matched no measurement cases"
    );
    if config.check_values || config.check_failures || config.check_reuse {
        assert!(
            config.checked.get() > 0,
            "filter matched no allocation contracts"
        );
    }
    assert_eq!(config.failures.get(), 0, "allocation contracts failed");
}
