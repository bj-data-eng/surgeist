//! Numbers carried by the preparation errors print in one form: plain for ordinary magnitudes and
//! exponent form beyond them, so that an extreme input never produces a message of hundreds of
//! characters.
//!
//! The expected texts are the shortest round-trip digits of each constant, written by hand:
//! `f64::MAX` is `1.7976931348623157e308`, `f64::MIN_POSITIVE` is `2.2250738585072014e-308`, and
//! the smallest subnormal is `5e-324`. A magnitude from 1e-5 up to but excluding 1e16 stays plain.

use surgeist_render::{
    convolution::ConvolutionParameter, convolution::ConvolutionPreparationError,
    displacement::DisplacementParameter, displacement::DisplacementPreparationError,
    gaussian::GaussianParameter, gaussian::GaussianPreparationError, geometry::Size,
    morphology::MorphologyParameter, morphology::MorphologyPreparationError,
    native_shadow::DropShadowPreparationError, offset::OffsetParameter,
    offset::OffsetPreparationError, text::GlyphPreparationError,
};

const EXTREMES: [(f64, &str); 5] = [
    (f64::MAX, "1.7976931348623157e308"),
    (f64::MIN, "-1.7976931348623157e308"),
    (f64::MIN_POSITIVE, "2.2250738585072014e-308"),
    (f64::from_bits(1), "5e-324"),
    (1e16, "1e16"),
];

const ORDINARY: [(f64, &str); 5] = [
    (0.0, "0"),
    (2.5, "2.5"),
    (-0.00001, "-0.00001"),
    (123_456_789.125, "123456789.125"),
    (9_999_999_999_999_998.0, "9999999999999998"),
];

fn each(build: impl Fn(f64) -> String, prefix: &str, suffix: &str) {
    for (value, text) in EXTREMES.into_iter().chain(ORDINARY) {
        let message = build(value);
        assert_eq!(message, format!("{prefix}{text}{suffix}"));
        assert!(message.len() < 120, "{message}");
    }
}

#[test]
fn filter_parameter_values_print_through_the_number_form() {
    each(
        |value| {
            ConvolutionPreparationError::InvalidParameter {
                parameter: ConvolutionParameter::Divisor,
                value,
            }
            .to_string()
        },
        "convolution parameter divisor has the invalid value ",
        "",
    );
    each(
        |value| {
            DisplacementPreparationError::InvalidParameter {
                parameter: DisplacementParameter::Scale,
                value,
            }
            .to_string()
        },
        "displacement parameter scale has the invalid value ",
        "",
    );
    each(
        |value| {
            GaussianPreparationError::InvalidParameter {
                parameter: GaussianParameter::StandardDeviationX,
                value,
            }
            .to_string()
        },
        "Gaussian parameter horizontal standard deviation has the invalid value ",
        "",
    );
    each(
        |value| {
            MorphologyPreparationError::InvalidParameter {
                parameter: MorphologyParameter::RadiusX,
                value,
            }
            .to_string()
        },
        "morphology parameter horizontal radius has the invalid value ",
        "",
    );
    each(
        |value| {
            OffsetPreparationError::InvalidParameter {
                parameter: OffsetParameter::Dx,
                value,
            }
            .to_string()
        },
        "offset parameter horizontal displacement has the invalid value ",
        "",
    );
}

#[test]
fn a_flood_opacity_prints_through_the_number_form() {
    each(
        |value| DropShadowPreparationError::InvalidFloodOpacity { value }.to_string(),
        "drop shadow flood opacity ",
        " is not finite or is outside 0 to 1",
    );
}

#[test]
fn a_displacement_map_size_prints_through_the_number_form() {
    let error = DisplacementPreparationError::InvalidMapDimensions {
        size: Size::try_new(1e300, 2.5).unwrap(),
    };
    assert_eq!(
        error.to_string(),
        "displacement map size 1e300 by 2.5 is not a pair of finite nonnegative integers that fit in u32"
    );
}

#[test]
fn a_single_precision_coordinate_prints_through_the_number_form() {
    for (value, text) in [
        (0.5_f32, "0.5"),
        (f32::MAX, "3.4028235e38"),
        (f32::MIN_POSITIVE, "1.1754944e-38"),
        (1e-6_f32, "1e-6"),
        (16384.5_f32, "16384.5"),
    ] {
        assert_eq!(
            GlyphPreparationError::CoordinateLoss { value }.to_string(),
            format!("variation coordinate {text} would change during quantization")
        );
    }
}

#[cfg(feature = "render-paint")]
#[test]
fn a_gradient_basis_prints_through_the_number_form() {
    use surgeist_render::paint_gradient::PaintGradientError;
    assert_eq!(
        PaintGradientError::BasisMismatch {
            expected: f64::MAX,
            actual: 0.000001,
        }
        .to_string(),
        "the used stops are measured on a line of length 1e-6, but the geometry's basis is 1.7976931348623157e308"
    );
}
