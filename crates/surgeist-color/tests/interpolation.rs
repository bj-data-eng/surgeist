use surgeist_color::{
    Color, ColorSpace, Computation, Finite, HueInterpolation, InterpolationComponent,
    InterpolationMethod, NumericalRequest, OperationOwner, PredefinedColorSpace, UnitInterval,
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

#[test]
fn sample_extrapolates_premultiplied_channels_and_alpha_in_both_directions() {
    let first = color(SRGB, [Some(0.25), Some(0.75), None], Some(0.25));
    let second = color(SRGB, [Some(0.75), Some(0.25), None], Some(0.75));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // Alpha is 1/4 + t/2. The premultiplied channels are 1/16 + t/2 and 3/16.
    for (progress, expected_alpha, expected_channels) in [
        (-1.0, -0.25, [Some(1.75), Some(-0.75), None]),
        (3.5, 2.0, [Some(0.90625), Some(0.09375), None]),
    ] {
        let sample = first
            .sample_interpolation(second, method, Finite::new(progress).unwrap())
            .unwrap();
        assert_eq!(sample.space(), SRGB);
        assert_eq!(sample.alpha().map(Finite::get), Some(expected_alpha));
        assert_eq!(
            sample.channels().map(|channel| channel.map(Finite::get)),
            expected_channels
        );
    }
}

#[test]
fn sample_at_zero_alpha_retains_the_premultiplied_coordinates() {
    let first = color(SRGB, [Some(0.25), Some(0.75), None], Some(0.25));
    let second = color(SRGB, [Some(0.75), Some(0.25), None], Some(0.75));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // Weights 3/2 and -1/2 cancel the alpha and leave -3/16 and 3/16.
    let sample = first
        .sample_interpolation(second, method, Finite::new(-0.5).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(0.0));
    assert_eq!(
        sample.channels().map(|channel| channel.map(Finite::get)),
        [Some(-0.1875), Some(0.1875), None]
    );
}

#[test]
fn sample_carries_one_missing_alpha_and_preserves_two_missing_alphas() {
    let first = color(SRGB, [Some(0.25), Some(0.75), None], None);
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let progress = Finite::new(2.0).unwrap();

    // Equal carried alphas cancel. Two missing alphas use straight coordinates.
    // In both cases, advancing two endpoint differences gives 5/4 and -1/4.
    for alpha in [Some(0.75), None] {
        let second = color(SRGB, [Some(0.75), Some(0.25), None], alpha);
        let sample = first
            .sample_interpolation(second, method, progress)
            .unwrap();
        assert_eq!(sample.alpha().map(Finite::get), alpha);
        assert_eq!(
            sample.channels().map(|channel| channel.map(Finite::get)),
            [Some(1.25), Some(-0.25), None]
        );
    }
}

#[test]
fn sample_extrapolates_the_selected_hue_arc() {
    let space = ColorSpace::Oklch;
    let first = color(space, [Some(0.5), Some(0.25), Some(30.0)], Some(0.25));
    let progress = Finite::new(1.25).unwrap();

    // The positive arcs cover 60 and 240 degrees. The negative arcs cover
    // -300 and -120 degrees. Advancing 5/4 of an arc wraps into one revolution.
    for (strategy, near_expected, far_expected) in [
        (HueInterpolation::Shorter, 105.0, 240.0),
        (HueInterpolation::Longer, 15.0, 330.0),
        (HueInterpolation::Increasing, 105.0, 330.0),
        (HueInterpolation::Decreasing, 15.0, 240.0),
    ] {
        let method = InterpolationMethod::try_new(space, Some(strategy)).unwrap();
        for (end_hue, expected) in [(90.0, near_expected), (270.0, far_expected)] {
            let second = color(space, [Some(0.5), Some(0.25), Some(end_hue)], Some(0.75));
            let sample = first
                .sample_interpolation(second, method, progress)
                .unwrap();
            assert_eq!(sample.channels()[2].map(Finite::get), Some(expected));
        }
    }
}

#[test]
fn sample_wraps_the_hue_at_the_largest_finite_progress_in_each_direction() {
    let space = ColorSpace::Oklch;
    let first = color(space, [Some(0.5), Some(0.25), Some(30.0)], Some(1.0));
    let second = color(space, [Some(0.5), Some(0.25), Some(90.0)], Some(1.0));
    let method = InterpolationMethod::try_new(space, Some(HueInterpolation::Shorter)).unwrap();

    // MAX is the integer (2^53 - 1) * 2^971, congruent to 2 modulo 6.
    // The 60-degree arc therefore travels +120 or -120 degrees modulo 360.
    for (progress, expected) in [(f64::MAX, 150.0), (-f64::MAX, 270.0)] {
        let sample = first
            .sample_interpolation(second, method, Finite::new(progress).unwrap())
            .unwrap();
        assert_eq!(sample.channels()[2].map(Finite::get), Some(expected));
    }
}

#[test]
fn sample_carries_individual_missing_analogues_after_conversion() {
    let first = color(ColorSpace::Lab, [None, Some(3.0), Some(4.0)], None);
    let second = color(ColorSpace::Lch, [Some(48.0), Some(13.0), None], None);
    let method = InterpolationMethod::try_new(ColorSpace::Lch, None).unwrap();

    // The Lab axes form a 3-4-5 triangle. The first endpoint supplies the hue,
    // the second supplies lightness, and chroma moves halfway from 5 away from 13.
    let sample = first
        .sample_interpolation(second, method, Finite::new(-0.5).unwrap())
        .unwrap();
    assert_eq!(sample.space(), ColorSpace::Lch);
    let [lightness, chroma, hue] = sample.channels().map(|channel| channel.map(Finite::get));
    assert_eq!(lightness, Some(48.0));
    assert_eq!(chroma, Some(1.0));
    let hue = hue.unwrap();
    assert!((hue - 53.130_102_354_155_98).abs() < 1e-12, "{hue}");
}

#[test]
fn sample_carries_missing_analogue_sets_and_preserves_a_shared_missing_channel() {
    let first = color(ColorSpace::Lab, [Some(50.0), None, None], None);
    let second = color(ColorSpace::Lch, [Some(74.0), Some(13.0), None], None);
    let method = InterpolationMethod::try_new(ColorSpace::Lch, None).unwrap();

    // The missing Lab axes carry as missing chroma and hue. The second
    // endpoint supplies chroma throughout, and lightness advances by 5/4 of 24.
    let sample = first
        .sample_interpolation(second, method, Finite::new(1.25).unwrap())
        .unwrap();
    assert_eq!(
        sample.channels().map(|channel| channel.map(Finite::get)),
        [Some(80.0), Some(13.0), None]
    );
}

#[test]
fn sample_divides_a_wide_premultiplied_coordinate_before_narrowing() {
    let high = 2.0_f64.powi(1023);
    let first = color(SRGB, [Some(f64::MAX), Some(0.0), None], Some(0.0));
    let second = color(SRGB, [Some(f64::MAX), Some(high), None], Some(1.0));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // The first endpoint contributes zero. Both coordinates of the second are
    // doubled beyond binary64, then division by the sampled alpha recovers them.
    let sample = first
        .sample_interpolation(second, method, Finite::new(2.0).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(2.0));
    assert_eq!(
        sample.channels().map(|channel| channel.map(Finite::get)),
        [Some(f64::MAX), Some(high), None]
    );
}

#[test]
fn sample_divides_by_the_exact_nonzero_alpha_after_near_cancellation() {
    let first = color(SRGB, [Some(1.0), None, None], Some(0.5));
    let second = color(SRGB, [Some(1.0), None, None], Some(0.125));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // Binary64 4/3 falls below the exact fraction by 2^-52/3. The alpha
    // slope is -3/8, so alpha is 2^-55 and a constant coordinate stays at 1.
    let sample = first
        .sample_interpolation(second, method, Finite::new(4.0 / 3.0).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(2.0_f64.powi(-55)));
    assert_eq!(sample.channels()[0].map(Finite::get), Some(1.0));
}

#[test]
fn sample_recovers_a_finite_coordinate_after_wide_weights_cancel() {
    let first_coordinate = 2.0;
    let second_coordinate = 2.0 + 2.0 * f64::EPSILON;
    let alpha = f64::from_bits(1);
    let first = color(SRGB, [Some(first_coordinate), None, None], Some(alpha));
    let second = color(SRGB, [Some(second_coordinate), None, None], Some(alpha));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // Equal alphas cancel. The endpoint difference is 2^-51. At MAX progress
    // the intercept 2 is too small to affect rounding of MAX * 2^-51.
    let sample = first
        .sample_interpolation(second, method, Finite::new(f64::MAX).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(alpha));
    assert_eq!(
        sample.channels()[0].map(Finite::get),
        Some(f64::MAX * 2.0_f64.powi(-51))
    );
}

#[test]
fn sample_reports_the_overflowing_channel_and_a_later_sample_succeeds() {
    let magnitude = 2.0_f64.powi(1023);
    let first = color(SRGB, [None, Some(-magnitude), Some(0.125)], Some(1.0));
    let second = color(SRGB, [None, Some(magnitude), Some(0.375)], Some(1.0));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // At t = 2 the second channel is 3 * 2^1023. At t = 3/4 it is 2^1022.
    let error = first
        .sample_interpolation(second, method, Finite::new(2.0).unwrap())
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Channel(1));

    let sample = first
        .sample_interpolation(second, method, Finite::new(0.75).unwrap())
        .unwrap();
    assert_eq!(
        sample.channels().map(|channel| channel.map(Finite::get)),
        [None, Some(2.0_f64.powi(1022)), Some(0.3125)]
    );
}

#[test]
fn sample_keeps_a_maximum_constant_channel_with_different_alphas() {
    let first = color(SRGB, [Some(f64::MAX), None, None], Some(1.0));
    let second = color(SRGB, [Some(f64::MAX), None, None], Some(0.75));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    let sample = first
        .sample_interpolation(second, method, Finite::new(-1.0).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(1.25));
    assert_eq!(sample.channels()[0].map(Finite::get), Some(f64::MAX));
}

#[test]
fn sample_keeps_low_bits_in_the_alpha_slope() {
    let below_half = f64::from_bits(0.5_f64.to_bits() - 1);
    let first = color(SRGB, [Some(0.25), None, None], Some(1.0));
    let second = color(SRGB, [Some(0.5), None, None], Some(below_half));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // At t = 2, alpha is 2 * below_half - 1 = -2^-53. The premultiplied
    // coordinate is below_half - .25, so division gives .5 - 2^51.
    let sample = first
        .sample_interpolation(second, method, Finite::new(2.0).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(-2.0_f64.powi(-53)));
    assert_eq!(
        sample.channels()[0].map(Finite::get),
        Some(0.5 - 2.0_f64.powi(51))
    );
}

#[test]
fn sample_keeps_a_tiny_hue_endpoint_at_extreme_progress() {
    let space = ColorSpace::Oklch;
    let first = color(space, [Some(0.5), Some(0.25), Some(2.0)], Some(1.0));
    let progress = 2.0_f64.powi(1023);
    let second = color(
        space,
        [Some(0.5), Some(0.25), Some(1.0 / progress)],
        Some(1.0),
    );
    let method = InterpolationMethod::try_new(space, Some(HueInterpolation::Decreasing)).unwrap();

    // 2 + (2^-1023 - 2) * 2^1023 = 3 - 2^1024. The power of two is
    // 16 modulo 360, so the canonical angle is 347 degrees.
    let sample = first
        .sample_interpolation(second, method, Finite::new(progress).unwrap())
        .unwrap();
    assert_eq!(sample.channels()[2].map(Finite::get), Some(347.0));
}

#[test]
fn longer_hue_distinguishes_a_near_semicircle_from_a_tie() {
    let space = ColorSpace::Oklch;
    let first = color(
        space,
        [Some(0.5), Some(0.25), Some(2.0_f64.powi(-54))],
        Some(1.0),
    );
    let second = color(space, [Some(0.5), Some(0.25), Some(180.0)], Some(1.0));
    let method = InterpolationMethod::try_new(space, Some(HueInterpolation::Longer)).unwrap();

    // The forward arc is just under 180 degrees, so the longer arc goes
    // backwards. One and a half negative semicircles wraps to 90 degrees.
    let sample = first
        .sample_interpolation(second, method, Finite::new(1.5).unwrap())
        .unwrap();
    assert_eq!(sample.channels()[2].map(Finite::get), Some(90.0));
}
