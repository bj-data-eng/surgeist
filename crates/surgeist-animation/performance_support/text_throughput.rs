//! Additive API adoption measurements, kept separate from legacy comparisons.

use std::hint::black_box;
use std::sync::Arc;

use surgeist_animation::{
    AnimationDelay, AnimationDuration, AnimationPlayState, CompositeValue, DiscreteValue,
    PropertyKey, TransformValue, TransitionTrack, TransitionTrackId,
};

use crate::{Config, lifecycle, measure, timed};

pub fn run(config: &Config) {
    let shared: Arc<str> = Arc::from("property_000");
    macro_rules! intake {
        ($name:literal, $operation:expr) => {
            measure(config, concat!("adoption/intake/", $name), |iterations| {
                timed(iterations, |_| {
                    black_box($operation);
                })
            });
        };
    }
    intake!(
        "property/legacy",
        PropertyKey::new(black_box("property_000")).unwrap()
    );
    intake!(
        "property/text",
        PropertyKey::from_text(black_box("property_000")).unwrap()
    );
    intake!(
        "property/shared",
        PropertyKey::from_shared(black_box(shared.clone())).unwrap()
    );
    intake!(
        "discrete/legacy",
        DiscreteValue::new(black_box("property_000")).unwrap()
    );
    intake!(
        "discrete/text",
        DiscreteValue::from_text(black_box("property_000")).unwrap()
    );
    intake!(
        "discrete/shared",
        DiscreteValue::from_shared(black_box(shared.clone())).unwrap()
    );
    intake!(
        "transform/legacy",
        TransformValue::unsupported(black_box("property_000"))
    );
    intake!(
        "transform/text",
        TransformValue::unsupported_from_text(black_box("property_000"))
    );
    intake!(
        "transform/shared",
        TransformValue::unsupported_from_shared(black_box(shared.clone()))
    );
    intake!(
        "composite/legacy",
        CompositeValue::unsupported(black_box("property_000"))
    );
    intake!(
        "composite/text",
        CompositeValue::unsupported_from_text(black_box("property_000"))
    );
    intake!(
        "composite/shared",
        CompositeValue::unsupported_from_shared(black_box(shared.clone()))
    );

    for family in lifecycle::LIFECYCLE_FAMILIES {
        for curve in lifecycle::Curve::ALL {
            for count in [0, 4, 60] {
                let times = lifecycle::sample_times(count);
                measure(
                    config,
                    &format!(
                        "adoption/transition/{}/{}/n{count}",
                        family.name(),
                        curve.name()
                    ),
                    |iterations| {
                        timed(iterations, |_| {
                            let (from, to) = lifecycle::endpoints(black_box(family), 0.0);
                            let track = TransitionTrack::new(
                                TransitionTrackId::new(1),
                                PropertyKey::from_text(black_box("property_000")).unwrap(),
                                from,
                                to,
                                AnimationDuration::from_secs(0.2).unwrap(),
                                AnimationDelay::from_secs(0.0).unwrap(),
                                black_box(curve).easing(),
                            )
                            .unwrap();
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
}
