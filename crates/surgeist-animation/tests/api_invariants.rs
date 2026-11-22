#![forbid(unsafe_code)]

use surgeist_animation::easing::{
    EasingError, LinearControlPointComponent, LinearControlPointInput, LinearEasing,
};
use surgeist_animation::progress::{EasingInput, NormalizedProgress};

#[test]
fn raw_linear_coordinates_report_the_rejected_index_and_component() {
    for (input, output, component, rejected) in [
        (f64::NAN, 0.5, LinearControlPointComponent::Input, f64::NAN),
        (
            f64::NEG_INFINITY,
            0.5,
            LinearControlPointComponent::Input,
            f64::NEG_INFINITY,
        ),
        (
            0.5,
            f64::INFINITY,
            LinearControlPointComponent::Output,
            f64::INFINITY,
        ),
    ] {
        let error = LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput { input, output },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap_err();
        let EasingError::NonFiniteLinearControlPoint {
            index,
            component: actual,
            value,
        } = error
        else {
            panic!("expected coordinate validation error, got {error:?}");
        };
        assert_eq!(index, 1);
        assert_eq!(actual, component);
        assert_eq!(value.to_bits(), rejected.to_bits());
    }
}

#[test]
fn raw_linear_coordinates_preserve_the_first_validation_error() {
    let inputs = vec![
        LinearControlPointInput {
            input: 0.5,
            output: 0.0,
        },
        LinearControlPointInput {
            input: 0.25,
            output: f64::INFINITY,
        },
        LinearControlPointInput {
            input: f64::NAN,
            output: 1.0,
        },
    ];
    assert_eq!(
        LinearEasing::new(inputs),
        Err(EasingError::NonFiniteLinearControlPoint {
            index: 1,
            component: LinearControlPointComponent::Output,
            value: f64::INFINITY,
        }),
    );
    assert_eq!(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.5,
                output: 0.0
            },
            LinearControlPointInput {
                input: 0.25,
                output: 0.5
            },
        ]),
        Err(EasingError::DecreasingLinearControlPointInput {
            index: 1,
            previous_input: 0.5,
            input: 0.25,
        }),
    );
}

#[test]
fn validated_linear_points_preserve_coordinates_and_sampling() {
    let easing = LinearEasing::new(vec![
        LinearControlPointInput {
            input: -0.0,
            output: -0.0,
        },
        LinearControlPointInput {
            input: 0.5,
            output: -2.0,
        },
        LinearControlPointInput {
            input: 1.0,
            output: 3.0,
        },
    ])
    .unwrap();
    let coordinates: Vec<_> = easing
        .points()
        .iter()
        .map(|point| (point.input().to_bits(), point.output().to_bits()))
        .collect();
    assert_eq!(
        coordinates,
        vec![
            ((-0.0_f64).to_bits(), (-0.0_f64).to_bits()),
            (0.5_f64.to_bits(), (-2.0_f64).to_bits()),
            (1.0_f64.to_bits(), 3.0_f64.to_bits()),
        ]
    );
    assert_eq!(
        easing
            .evaluate(NormalizedProgress::new(0.25).unwrap())
            .value(),
        -1.0
    );
    assert_eq!(
        easing
            .evaluate_unrestricted(EasingInput::new(1.5).unwrap())
            .unwrap()
            .value(),
        8.0
    );
}
