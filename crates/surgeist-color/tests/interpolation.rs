use surgeist_color::{
    Color, ColorSpace, Computation, Finite, InterpolationMethod, NumericalRequest, OperationOwner,
    PredefinedColorSpace, UnitInterval,
};

const SRGB: ColorSpace = ColorSpace::Predefined(PredefinedColorSpace::Srgb);

fn color(space: ColorSpace, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
    Color::new(
        space,
        channels.map(|value| value.map(|value| Finite::new(value).unwrap())),
        alpha.map(|value| UnitInterval::new(value).unwrap()),
    )
}

#[test]
fn ordinary_interpolation_preserves_a_constant_subnormal_channel() {
    let minimum = f64::from_bits(1);
    let endpoint = color(SRGB, [Some(minimum), None, None], Some(1.0));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let midpoint = UnitInterval::new(0.5).unwrap();

    // A constant channel remains constant. Rounding each half before the sum
    // would erase both halves of the smallest positive coordinate.
    assert_eq!(
        endpoint.interpolated(endpoint, method, midpoint).channels(),
        [Some(minimum), None, None]
    );
}

#[test]
fn controlled_interpolation_retains_subnormal_alpha_and_its_color() {
    let minimum = f64::from_bits(1);
    let first = color(SRGB, [Some(0.25), None, None], Some(minimum));
    let second = color(SRGB, [Some(0.75), None, None], Some(minimum));
    let request = NumericalRequest::Interpolate {
        first,
        second,
        method: InterpolationMethod::try_new(SRGB, None).unwrap(),
        progress: UnitInterval::new(0.5).unwrap(),
    };
    let mut owner = OperationOwner::new();
    let value = Computation::new(&mut owner, request).compute();

    // Equal nonzero alpha cancels from the premultiplied mean. Neither the
    // weighted alpha nor the premultiplied color should narrow before division.
    assert_eq!(value.alpha(), Some(minimum));
    assert_eq!(value.channels(), [Some(0.5), None, None]);
}
