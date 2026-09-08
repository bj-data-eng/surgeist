//! Uninstrumented release timing. Construction measurements include disposal;
//! prepared construction excludes input cloning, performed before timed chunks.

#![forbid(unsafe_code)]

#[path = "../performance_support/fixtures.rs"]
mod fixtures;
#[path = "../performance_support/lifecycle.rs"]
mod lifecycle;
#[path = "../performance_support/text_throughput.rs"]
mod text_throughput;

use std::cell::Cell;
use std::hint::black_box;
use std::time::{Duration, Instant};
use surgeist_animation::{
    AnimationPlayState, CubicBezier, Easing, EasingInput, EasingKeyword, LinearControlPoint,
    LinearEasing, NormalizedProgress,
};

use fixtures::{Family, Layout, SIZES};
use lifecycle::{Curve, KeyReuse};

struct Config {
    filter: String,
    batch: Duration,
    warmup: Duration,
    batches: usize,
    matched: Cell<usize>,
}

impl Config {
    fn read() -> Self {
        let mut result = Self {
            filter: String::new(),
            batch: Duration::from_millis(25),
            warmup: Duration::from_millis(10),
            batches: 7,
            matched: Cell::new(0),
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bench" => {}
                "--filter" => result.filter = args.next().expect("--filter needs text"),
                "--batch-ms" => {
                    result.batch = Duration::from_millis(positive(&mut args, "--batch-ms"));
                }
                "--warmup-ms" => {
                    result.warmup = Duration::from_millis(positive(&mut args, "--warmup-ms"));
                }
                "--batches" => {
                    result.batches = usize::try_from(positive(&mut args, "--batches")).unwrap();
                    assert!(result.batches >= 3, "at least three batches are required");
                }
                "--help" => {
                    println!(
                        "sampling [--filter SUBSTRING] [--batch-ms 25] [--warmup-ms 10] [--batches 7]"
                    );
                    std::process::exit(0);
                }
                _ => panic!("unknown argument: {arg}"),
            }
        }
        result
    }
}

fn positive(args: &mut impl Iterator<Item = String>, flag: &str) -> u64 {
    let value: u64 = args
        .next()
        .unwrap_or_else(|| panic!("{flag} needs a value"))
        .parse()
        .unwrap_or_else(|_| panic!("{flag} needs an integer"));
    assert!(value > 0, "{flag} must be positive");
    value
}

fn measure(config: &Config, name: &str, mut run: impl FnMut(usize) -> Duration) {
    if !name.contains(&config.filter) {
        return;
    }
    config.matched.set(config.matched.get() + 1);
    let warmup = Instant::now();
    let mut iterations = 1;
    let mut elapsed = run(iterations);
    while warmup.elapsed() < config.warmup {
        if elapsed < config.batch / 4 {
            iterations = (iterations * 2).min(16_777_216);
        }
        elapsed = run(iterations);
    }
    iterations =
        ((iterations as f64 * config.batch.as_secs_f64() / elapsed.as_secs_f64().max(1e-9))
            as usize)
            .clamp(1, 16_777_216);
    let mut values = Vec::with_capacity(config.batches);
    for _ in 0..config.batches {
        let elapsed = run(iterations);
        values.push(elapsed.as_secs_f64() * 1e9 / iterations as f64);
    }
    values.sort_by(f64::total_cmp);
    let median = if values.len() % 2 == 0 {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    };
    let min = values[0];
    let max = values[values.len() - 1];
    println!(
        "{name},{iterations},{},{median:.3},{min:.3},{max:.3}",
        config.batches
    );
}

#[inline(never)]
fn timed(iterations: usize, mut operation: impl FnMut(usize)) -> Duration {
    let start = Instant::now();
    for index in 0..iterations {
        operation(index);
    }
    start.elapsed()
}

fn measure_lifecycles(config: &Config) {
    for family in lifecycle::LIFECYCLE_FAMILIES {
        for curve in Curve::ALL {
            let clone_source = lifecycle::transition(family, curve);
            measure(
                config,
                &format!("clone/transition/{}/{}", family.name(), curve.name()),
                |iterations| {
                    timed(iterations, |_| {
                        black_box(black_box(&clone_source).clone());
                    })
                },
            );
            for sample_count in lifecycle::SAMPLE_COUNTS {
                let times = lifecycle::sample_times(sample_count);
                measure(
                    config,
                    &format!(
                        "lifecycle/transition/{}/{}/n{sample_count}",
                        family.name(),
                        curve.name()
                    ),
                    |iterations| {
                        timed(iterations, |_| {
                            let track = lifecycle::transition(black_box(family), black_box(curve));
                            for &elapsed in &times {
                                black_box(
                                    track.sample(black_box(elapsed), AnimationPlayState::Running),
                                );
                            }
                            black_box(track);
                        })
                    },
                );
            }
        }
    }

    for layout in Layout::ALL {
        for properties in lifecycle::PROPERTY_COUNTS {
            for per_property in lifecycle::SEGMENT_COUNTS {
                for key_reuse in KeyReuse::ALL {
                    for curve in Curve::ALL {
                        let suffix = format!(
                            "{}/p{properties}/s{per_property}/{}/{}",
                            layout.name(),
                            key_reuse.name(),
                            curve.name()
                        );
                        let construct = || {
                            lifecycle::track(lifecycle::segments(
                                black_box(properties),
                                black_box(per_property),
                                black_box(layout),
                                Family::Number,
                                black_box(key_reuse),
                                black_box(curve),
                            ))
                        };
                        let clone_source = construct();
                        measure(config, &format!("clone/keyframe/{suffix}"), |iterations| {
                            timed(iterations, |_| {
                                black_box(black_box(&clone_source).clone());
                            })
                        });
                        for sample_count in lifecycle::SAMPLE_COUNTS {
                            let times = lifecycle::sample_times(sample_count);
                            let suffix = format!("{suffix}/n{sample_count}");
                            measure(
                                config,
                                &format!("lifecycle/keyframe/sample/{suffix}"),
                                |iterations| {
                                    timed(iterations, |_| {
                                        let track = construct();
                                        for &elapsed in &times {
                                            black_box(track.sample(
                                                black_box(elapsed),
                                                AnimationPlayState::Running,
                                            ));
                                        }
                                        black_box(track);
                                    })
                                },
                            );
                            measure(
                                config,
                                &format!("lifecycle/keyframe/sample_into_fresh/{suffix}"),
                                |iterations| {
                                    timed(iterations, |_| {
                                        let track = construct();
                                        // Allocate on the first sample, so n0 remains
                                        // construction/disposal without an unused buffer.
                                        let mut output = Vec::new();
                                        for &elapsed in &times {
                                            track.sample_into(
                                                black_box(elapsed),
                                                AnimationPlayState::Running,
                                                &mut output,
                                            );
                                            black_box(output.as_slice());
                                        }
                                        drop(black_box(output));
                                        black_box(track);
                                    })
                                },
                            );
                            // This case excludes only output-buffer allocation/disposal.
                            // Every result is cleared before its track is dropped inside
                            // the measured lifecycle; no previous track stays retained.
                            let mut output = Vec::with_capacity(properties);
                            measure(
                                config,
                                &format!("lifecycle/keyframe/sample_into_persistent/{suffix}"),
                                |iterations| {
                                    timed(iterations, |_| {
                                        let track = construct();
                                        for &elapsed in &times {
                                            track.sample_into(
                                                black_box(elapsed),
                                                AnimationPlayState::Running,
                                                &mut output,
                                            );
                                            black_box(output.as_slice());
                                        }
                                        output.clear();
                                        black_box(track);
                                    })
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let config = Config::read();
    println!("case,iterations_per_batch,batches,median_ns,min_ns,max_ns");
    let elapsed = fixtures::elapsed_inputs();
    for layout in Layout::ALL {
        for properties in SIZES {
            for per_property in SIZES {
                let suffix = format!("{}/p{properties}/s{per_property}", layout.name());
                measure(
                    &config,
                    &format!("construct/complete/{suffix}"),
                    |iterations| {
                        timed(iterations, |_| {
                            black_box(fixtures::track(fixtures::segments(
                                black_box(properties),
                                black_box(per_property),
                                layout,
                                Family::Number,
                            )));
                        })
                    },
                );
                let prepared = fixtures::segments(properties, per_property, layout, Family::Number);
                measure(
                    &config,
                    &format!("construct/prepared/{suffix}"),
                    |iterations| {
                        let mut total = Duration::ZERO;
                        let mut remaining = iterations;
                        while remaining != 0 {
                            // Bounded prepared batches keep setup outside timing without
                            // retaining an unbounded number of large tracks in memory.
                            let count = remaining.min(16);
                            let inputs: Vec<_> = (0..count).map(|_| prepared.clone()).collect();
                            let start = Instant::now();
                            for input in inputs {
                                black_box(fixtures::track(black_box(input)));
                            }
                            total += start.elapsed();
                            remaining -= count;
                        }
                        total
                    },
                );
                let track = fixtures::track(prepared);
                measure(
                    &config,
                    &format!("keyframe/sample/{suffix}"),
                    |iterations| {
                        timed(iterations, |index| {
                            black_box(track.sample(
                                black_box(elapsed[index % elapsed.len()]),
                                AnimationPlayState::Running,
                            ));
                        })
                    },
                );
                let mut output = Vec::with_capacity(properties);
                track.sample_into(elapsed[0], AnimationPlayState::Running, &mut output);
                measure(
                    &config,
                    &format!("keyframe/sample_into/{suffix}"),
                    |iterations| {
                        timed(iterations, |index| {
                            track.sample_into(
                                black_box(elapsed[index % elapsed.len()]),
                                AnimationPlayState::Running,
                                &mut output,
                            );
                            black_box(output.as_slice());
                        })
                    },
                );
            }
        }
    }
    for family in Family::ALL {
        measure(
            &config,
            &format!("transition/construct/{}", family.name()),
            |iterations| {
                timed(iterations, |_| {
                    black_box(fixtures::transition(black_box(family)));
                })
            },
        );
        let transition = fixtures::transition(family);
        measure(
            &config,
            &format!("transition/sample/{}", family.name()),
            |iterations| {
                timed(iterations, |index| {
                    black_box(transition.sample(
                        black_box(elapsed[index % elapsed.len()]),
                        AnimationPlayState::Running,
                    ));
                })
            },
        );
        let track = fixtures::track(fixtures::segments(1, 1, Layout::Grouped, family));
        measure(
            &config,
            &format!("keyframe/sample/family/{}", family.name()),
            |iterations| {
                timed(iterations, |index| {
                    black_box(track.sample(
                        black_box(elapsed[index % elapsed.len()]),
                        AnimationPlayState::Running,
                    ));
                })
            },
        );
        let mut output = Vec::with_capacity(1);
        track.sample_into(elapsed[0], AnimationPlayState::Running, &mut output);
        measure(
            &config,
            &format!("keyframe/sample_into/family/{}", family.name()),
            |iterations| {
                timed(iterations, |index| {
                    track.sample_into(
                        black_box(elapsed[index % elapsed.len()]),
                        AnimationPlayState::Running,
                        &mut output,
                    );
                    black_box(output.as_slice());
                })
            },
        );
    }
    for count in [2, 16, 256] {
        let easing = LinearEasing::new(
            (0..count)
                .map(|index| {
                    let x = index as f64 / (count - 1) as f64;
                    LinearControlPoint::new(x, x * x)
                })
                .collect(),
        )
        .unwrap();
        let inputs = elapsed.map(|input| EasingInput::new(input.as_secs()).unwrap());
        measure(
            &config,
            &format!("linear/unrestricted/n{count}"),
            |iterations| {
                timed(iterations, |index| {
                    black_box(
                        easing
                            .evaluate_unrestricted(black_box(inputs[index % inputs.len()]))
                            .unwrap(),
                    );
                })
            },
        );
    }
    let curves = [
        ("ease", Easing::keyword(EasingKeyword::Ease)),
        ("ease_in", Easing::keyword(EasingKeyword::EaseIn)),
        ("ease_out", Easing::keyword(EasingKeyword::EaseOut)),
        ("ease_in_out", Easing::keyword(EasingKeyword::EaseInOut)),
        (
            "flat_start",
            Easing::cubic_bezier(CubicBezier::new(0.0, 0.2, 0.0, 0.8).unwrap()),
        ),
        (
            "flat_middle",
            Easing::cubic_bezier(CubicBezier::new(1.0, 0.2, 0.0, 0.8).unwrap()),
        ),
    ];
    let inputs = elapsed.map(|input| NormalizedProgress::new(input.as_secs()).unwrap());
    for (name, easing) in curves {
        measure(&config, &format!("cubic/normalized/{name}"), |iterations| {
            timed(iterations, |index| {
                black_box(easing.evaluate(black_box(inputs[index % inputs.len()])));
            })
        });
    }
    measure_lifecycles(&config);
    text_throughput::run(&config);
    assert!(
        config.matched.get() > 0,
        "filter matched no measurement cases"
    );
}
