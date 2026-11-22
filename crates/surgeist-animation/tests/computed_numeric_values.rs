use surgeist_animation::interpolation::{
    InterpolableNumber, InterpolablePercentage, InterpolableValue, InterpolationOutcome,
    InterpolationPair, InterpolationProgress, PropertyKey,
};

#[test]
fn opposite_finite_numbers_have_a_finite_midpoint() {
    let pair = pair("number", number(f64::MAX), number(-f64::MAX));
    assert_eq!(
        pair.sample(progress(0.5)),
        InterpolationOutcome::Value(number(0.0))
    );
}

#[test]
fn opposite_finite_percentages_have_a_finite_midpoint() {
    let pair = pair("percentage", percentage(f64::MAX), percentage(-f64::MAX));
    assert_eq!(
        pair.sample(progress(0.5)),
        InterpolationOutcome::Value(percentage(0.0))
    );
}

#[test]
fn an_unrepresentable_extrapolation_fails_and_the_endpoint_remains_usable() {
    let pair = pair("number", number(0.0), number(f64::MAX));
    assert!(matches!(
        pair.sample(progress(2.0)),
        InterpolationOutcome::ArithmeticFailure(_)
    ));
    assert_eq!(
        pair.sample(progress(1.0)),
        InterpolationOutcome::Value(number(f64::MAX))
    );
}

fn pair(property: &str, from: InterpolableValue, to: InterpolableValue) -> InterpolationPair {
    InterpolationPair::new(PropertyKey::from_text(property).unwrap(), from, to).unwrap()
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn percentage(value: f64) -> InterpolableValue {
    InterpolableValue::percentage(InterpolablePercentage::new(value).unwrap())
}

fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
