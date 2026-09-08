//! Shared external-consumer inputs for throughput and allocation measurements.

use surgeist_animation::{
    AnimationDelay, AnimationDirection, AnimationDuration, ColorInterpolationSpace, CompositeValue,
    DiscreteValue, Easing, ElapsedTime, FillMode, InterpolableColor, InterpolableNumber,
    InterpolablePercentage, InterpolableValue, IterationCount, KeyframeAnimationId,
    KeyframeSegment, KeyframeTiming, KeyframeTrack, PropertyKey, TransformValue, TransitionTrack,
    TransitionTrackId, UnitRatio,
};

pub const SIZES: [usize; 3] = [1, 8, 64];

#[derive(Clone, Copy)]
pub enum Layout {
    Grouped,
    Interleaved,
}

impl Layout {
    pub const ALL: [Self; 2] = [Self::Grouped, Self::Interleaved];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Grouped => "grouped",
            Self::Interleaved => "interleaved",
        }
    }
}

#[derive(Clone, Copy)]
pub enum Family {
    Number,
    Percentage,
    Color,
    Discrete,
    Transform,
    Composite,
    ArithmeticFailure,
}

impl Family {
    pub const ALL: [Self; 7] = [
        Self::Number,
        Self::Percentage,
        Self::Color,
        Self::Discrete,
        Self::Transform,
        Self::Composite,
        Self::ArithmeticFailure,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Percentage => "percentage",
            Self::Color => "color",
            Self::Discrete => "discrete",
            Self::Transform => "transform",
            Self::Composite => "composite",
            Self::ArithmeticFailure => "arithmetic_failure",
        }
    }

    fn endpoints(self, bias: f64) -> (InterpolableValue, InterpolableValue) {
        match self {
            Self::Number => (number(10.0 + bias), number(30.0 + bias)),
            Self::Percentage => (
                InterpolableValue::percentage(InterpolablePercentage::new(-0.5).unwrap()),
                InterpolableValue::percentage(InterpolablePercentage::new(1.5).unwrap()),
            ),
            Self::Color => (color(0.2, 0.4, 0.6), color(0.8, 0.6, 0.4)),
            Self::Discrete => (
                InterpolableValue::discrete(DiscreteValue::new("hidden").unwrap()),
                InterpolableValue::discrete(DiscreteValue::new("visible").unwrap()),
            ),
            Self::Transform => (
                InterpolableValue::transform(TransformValue::unsupported("translate")),
                InterpolableValue::transform(TransformValue::unsupported("rotate")),
            ),
            Self::Composite => (
                InterpolableValue::composite(CompositeValue::unsupported("shadow-a")),
                InterpolableValue::composite(CompositeValue::unsupported("shadow-b")),
            ),
            Self::ArithmeticFailure => (number(f64::MAX), number(-f64::MAX)),
        }
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

pub fn timing() -> KeyframeTiming {
    KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap()
}

pub fn segments(
    properties: usize,
    per_property: usize,
    layout: Layout,
    family: Family,
) -> Vec<KeyframeSegment> {
    let mut result = Vec::with_capacity(properties * per_property);
    for index in 0..properties * per_property {
        let (property, segment) = match layout {
            Layout::Grouped => (index / per_property, index % per_property),
            Layout::Interleaved => (index % properties, index / properties),
        };
        let (from, to) = family.endpoints(property as f64 + segment as f64 * 2.0);
        result.push(
            KeyframeSegment::new(
                PropertyKey::new(format!("property_{property:03}")).unwrap(),
                UnitRatio::new(segment as f64 / per_property as f64).unwrap(),
                UnitRatio::new((segment + 1) as f64 / per_property as f64).unwrap(),
                from,
                to,
                Easing::linear(),
            )
            .unwrap(),
        );
    }
    result
}

pub fn track(segments: Vec<KeyframeSegment>) -> KeyframeTrack {
    KeyframeTrack::new(
        KeyframeAnimationId::new("measurement").unwrap(),
        timing(),
        segments,
    )
    .unwrap()
}

pub fn transition(family: Family) -> TransitionTrack {
    let (from, to) = family.endpoints(0.0);
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("property_000").unwrap(),
        from,
        to,
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        Easing::linear(),
    )
    .unwrap()
}

/// A full-period permutation of 1,024 interior sample times. The irrational
/// fraction keeps inputs away from exact segment and easing control boundaries.
pub fn elapsed_inputs() -> [ElapsedTime; 1024] {
    std::array::from_fn(|index| {
        ElapsedTime::from_secs((((index * 613) % 1024) as f64 + 0.381_966) / 1024.0).unwrap()
    })
}
