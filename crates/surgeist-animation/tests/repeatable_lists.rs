#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedComponent, ComputedErrorKind, ComputedNumeric, ComputedValue, RepeatableList,
};
use surgeist_animation::interpolation::{DiscreteValue, InterpolationProgress};

fn item(value: f64) -> ComputedValue {
    ComputedValue::single(ComputedComponent::Numeric(
        ComputedNumeric::number(value).unwrap(),
    ))
}
fn list(values: &[f64]) -> RepeatableList {
    RepeatableList::new(values.iter().copied().map(item).collect()).unwrap()
}
fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}

#[test]
fn different_lengths_repeat_to_the_lcm_before_combination() {
    let first = list(&[0.0, 20.0]);
    let second = list(&[10.0, 30.0, 50.0]);
    assert_eq!(
        first.interpolate(&second, progress(0.5)).unwrap(),
        list(&[5.0, 25.0, 25.0, 15.0, 15.0, 35.0])
    );
    assert_eq!(
        first.add(&second).unwrap(),
        list(&[10.0, 50.0, 50.0, 30.0, 30.0, 70.0])
    );
    assert_eq!(
        first.accumulate(&second).unwrap(),
        first.add(&second).unwrap()
    );
}

#[test]
fn any_discrete_component_falls_back_to_the_original_whole_list() {
    let value = |number| {
        ComputedValue::new(vec![
            ComputedComponent::Numeric(ComputedNumeric::number(number).unwrap()),
            ComputedComponent::Discrete(DiscreteValue::new("same").unwrap()),
        ])
        .unwrap()
    };
    let first = RepeatableList::new(vec![value(0.0)]).unwrap();
    let second = RepeatableList::new(vec![value(10.0), value(20.0)]).unwrap();
    // Equal discrete components permit numeric interpolation by computed value.
    assert_ne!(
        value(0.0)
            .interpolate(&value(10.0), progress(0.25))
            .unwrap(),
        value(0.0)
    );
    assert_eq!(first.interpolate(&second, progress(0.499)).unwrap(), first);
    assert_eq!(first.interpolate(&second, progress(0.5)).unwrap(), second);
    assert_eq!(first.add(&second).unwrap(), second);
}

#[test]
fn incompatible_items_choose_original_endpoints_even_with_extrapolation() {
    let first = list(&[1.0, 2.0]);
    let second = RepeatableList::new(vec![ComputedValue::single(ComputedComponent::Numeric(
        ComputedNumeric::percentage(10.0).unwrap(),
    ))])
    .unwrap();
    assert_eq!(first.interpolate(&second, progress(-1.0)).unwrap(), first);
    assert_eq!(first.interpolate(&second, progress(2.0)).unwrap(), second);
    assert_eq!(first.accumulate(&second).unwrap(), second);
}

#[test]
fn empty_lists_are_rejected_and_expansion_limits_are_checked() {
    assert_eq!(
        RepeatableList::new(vec![]).unwrap_err().kind(),
        ComputedErrorKind::EmptyList
    );
    assert_eq!(RepeatableList::combined_len(2, 3).unwrap(), 6);
    assert_eq!(RepeatableList::combined_len(4, 4).unwrap(), 4);
    assert_eq!(
        RepeatableList::combined_len(0, 3).unwrap_err().kind(),
        ComputedErrorKind::EmptyList
    );
    assert_eq!(
        RepeatableList::combined_len(usize::MAX, 2)
            .unwrap_err()
            .kind(),
        ComputedErrorKind::ListExpansion
    );
}

#[test]
fn arithmetic_failure_is_distinct_from_incompatibility_and_preserves_inputs() {
    let first = list(&[0.0, f64::MAX]);
    let second = list(&[2.0, f64::MAX]);
    let saved = first.clone();
    assert_eq!(
        first.add(&second).unwrap_err().kind(),
        ComputedErrorKind::NonFiniteResult
    );
    assert_eq!(first, saved);
}

#[test]
fn later_structural_mismatch_wins_over_an_earlier_arithmetic_failure() {
    let first = list(&[f64::MAX, 2.0]);
    let second = RepeatableList::new(vec![
        item(f64::MAX),
        ComputedValue::single(ComputedComponent::Numeric(
            ComputedNumeric::percentage(4.0).unwrap(),
        )),
    ])
    .unwrap();
    assert_eq!(first.add(&second).unwrap(), second);
}

#[test]
fn repeatable_values_sample_through_the_existing_interpolation_contract() {
    use surgeist_animation::interpolation::{
        InterpolableValue, InterpolationOutcome, InterpolationPair, PropertyKey,
    };
    let pair = InterpolationPair::new(
        PropertyKey::new("--list").unwrap(),
        InterpolableValue::repeatable_list(list(&[0.0, 20.0])),
        InterpolableValue::repeatable_list(list(&[10.0, 30.0, 50.0])),
    )
    .unwrap();
    assert_eq!(
        pair.sample(progress(0.5)),
        InterpolationOutcome::Value(InterpolableValue::repeatable_list(list(&[
            5.0, 25.0, 25.0, 15.0, 15.0, 35.0
        ])))
    );
}
