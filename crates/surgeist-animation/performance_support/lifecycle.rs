//! Complete 200 ms lifecycle inputs, separate from the original sampling fixtures.
//! Caller keys, values, and tracks are created by these helpers at the measured
//! boundary. Only elapsed sample times and persistent output capacity are prepared.

use surgeist_animation::{
    AnimationDelay, AnimationDirection, AnimationDuration, ColorInterpolationSpace, CompositeValue,
    DiscreteValue, Easing, EasingKeyword, ElapsedTime, FillMode, InterpolableColor,
    InterpolableNumber, InterpolablePercentage, InterpolableValue, IterationCount,
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack, PropertyKey,
    TransformValue, TransitionTrack, TransitionTrackId, UnitRatio,
};

use crate::fixtures::{Family, Layout};

#[allow(
    dead_code,
    reason = "Only the throughput target enumerates sample counts."
)]
pub const SAMPLE_COUNTS: [usize; 5] = [0, 1, 4, 12, 60];
pub const PROPERTY_COUNTS: [usize; 3] = [1, 2, 8];
pub const SEGMENT_COUNTS: [usize; 4] = [1, 2, 8, 64];
#[allow(
    dead_code,
    reason = "The allocation target reports all value families instead."
)]
pub const LIFECYCLE_FAMILIES: [Family; 4] = [
    Family::Number,
    Family::Percentage,
    Family::Color,
    Family::Discrete,
];

#[derive(Clone, Copy)]
pub enum Curve {
    Linear,
    #[allow(dead_code, reason = "The allocation target uses linear timing only.")]
    Ease,
}

impl Curve {
    #[allow(
        dead_code,
        reason = "Only the throughput target enumerates easing curves."
    )]
    pub const ALL: [Self; 2] = [Self::Linear, Self::Ease];

    #[allow(dead_code, reason = "Only throughput rows distinguish easing names.")]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Ease => "ease",
        }
    }

    pub fn easing(self) -> Easing {
        match self {
            Self::Linear => Easing::linear(),
            Self::Ease => Easing::keyword(EasingKeyword::Ease),
        }
    }
}

#[derive(Clone, Copy)]
pub enum KeyReuse {
    PerSegment,
    PerProperty,
}

impl KeyReuse {
    pub const ALL: [Self; 2] = [Self::PerSegment, Self::PerProperty];

    pub const fn name(self) -> &'static str {
        match self {
            Self::PerSegment => "keys_fresh_per_segment",
            Self::PerProperty => "keys_reused_per_property",
        }
    }
}

#[allow(
    dead_code,
    reason = "The allocation target samples fixed observation points."
)]
pub fn sample_times(count: usize) -> Vec<ElapsedTime> {
    match count {
        0 => Vec::new(),
        1 => vec![ElapsedTime::from_secs(0.1).unwrap()],
        _ => (0..count)
            .map(|index| ElapsedTime::from_secs(0.2 * (index as f64 / (count - 1) as f64)).unwrap())
            .collect(),
    }
}

pub fn endpoints(family: Family, bias: f64) -> (InterpolableValue, InterpolableValue) {
    match family {
        Family::Number => (number(10.0 + bias), number(30.0 + bias)),
        Family::Percentage => (
            InterpolableValue::percentage(InterpolablePercentage::new(-0.5).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(1.5).unwrap()),
        ),
        Family::Color => (color(0.2, 0.4, 0.6), color(0.8, 0.6, 0.4)),
        Family::Discrete => (
            InterpolableValue::discrete(DiscreteValue::new("hidden").unwrap()),
            InterpolableValue::discrete(DiscreteValue::new("visible").unwrap()),
        ),
        Family::Transform => (
            InterpolableValue::transform(TransformValue::unsupported("translate")),
            InterpolableValue::transform(TransformValue::unsupported("rotate")),
        ),
        Family::Composite => (
            InterpolableValue::composite(CompositeValue::unsupported("shadow-a")),
            InterpolableValue::composite(CompositeValue::unsupported("shadow-b")),
        ),
        Family::ArithmeticFailure => (number(f64::MAX), number(-f64::MAX)),
    }
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn color(red: f64, green: f64, blue: f64) -> InterpolableValue {
    InterpolableValue::color(
        InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Srgb,
            red,
            green,
            blue,
            UnitRatio::new(1.0).unwrap(),
        )
        .unwrap(),
    )
}

pub fn animation_id() -> KeyframeAnimationId {
    KeyframeAnimationId::new("measurement").unwrap()
}

pub fn timing() -> KeyframeTiming {
    KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(0.2).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap()
}

pub fn transition(family: Family, curve: Curve) -> TransitionTrack {
    let (from, to) = endpoints(family, 0.0);
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("property_000").unwrap(),
        from,
        to,
        AnimationDuration::from_secs(0.2).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        curve.easing(),
    )
    .unwrap()
}

pub fn segments(
    properties: usize,
    per_property: usize,
    layout: Layout,
    family: Family,
    key_reuse: KeyReuse,
    curve: Curve,
) -> Vec<KeyframeSegment> {
    let shared_keys: Vec<_> = match key_reuse {
        KeyReuse::PerSegment => Vec::new(),
        KeyReuse::PerProperty => (0..properties)
            .map(|property| PropertyKey::new(format!("property_{property:03}")).unwrap())
            .collect(),
    };
    let mut result = Vec::with_capacity(properties * per_property);
    for index in 0..properties * per_property {
        let (property, segment) = match layout {
            Layout::Grouped => (index / per_property, index % per_property),
            Layout::Interleaved => (index % properties, index / properties),
        };
        let key = match key_reuse {
            KeyReuse::PerSegment => PropertyKey::new(format!("property_{property:03}")).unwrap(),
            KeyReuse::PerProperty => shared_keys[property].clone(),
        };
        let (from, to) = endpoints(family, property as f64 + segment as f64 * 2.0);
        result.push(
            KeyframeSegment::new(
                key,
                UnitRatio::new(segment as f64 / per_property as f64).unwrap(),
                UnitRatio::new((segment + 1) as f64 / per_property as f64).unwrap(),
                from,
                to,
                curve.easing(),
            )
            .unwrap(),
        );
    }
    result
}

pub fn track(segments: Vec<KeyframeSegment>) -> KeyframeTrack {
    KeyframeTrack::new(animation_id(), timing(), segments).unwrap()
}
