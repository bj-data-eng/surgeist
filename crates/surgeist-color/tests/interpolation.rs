use surgeist_color::{
    Color, ColorSpace, Computation, Finite, HueInterpolation, InterpolationComponent,
    InterpolationEndpoint, InterpolationMethod, NumericalRequest, OperationOwner,
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
fn sampled_endpoint_distinguishes_zero_alpha_residue_from_rounded_zero() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let residue = color(SRGB, [Some(0.75), None, None], Some(0.5))
        .sample_interpolation(
            color(SRGB, [Some(0.5), None, None], Some(1.0)),
            method,
            Finite::new(-1.0).unwrap(),
        )
        .unwrap();
    let transparent = color(SRGB, [Some(0.25), None, None], Some(0.0));
    let rounded_zero = color(SRGB, [Some(0.25), None, None], Some(f64::from_bits(1)))
        .sample_interpolation(transparent, method, Finite::new(0.5).unwrap())
        .unwrap();
    let target = color(SRGB, [Some(0.5), None, None], Some(1.0));

    // The first alpha is exactly zero and leaves a numerator of 1/4.
    // The second alpha is half the minimum subnormal and rounds to zero
    // after division has already returned a straight coordinate of 1/4.
    assert_eq!(residue.channels(), rounded_zero.channels());
    assert_eq!(residue.alpha(), Finite::new(0.0));
    assert_eq!(rounded_zero.alpha(), Finite::new(0.0));
    assert!(!residue.equivalent_to_resolved(transparent));
    assert!(rounded_zero.equivalent_to_resolved(transparent));
    for (sample, expected) in [(residue, 0.75), (rounded_zero, 0.5)] {
        let output = InterpolationEndpoint::Sampled(sample)
            .sample_interpolation(
                InterpolationEndpoint::Resolved(target),
                method,
                Finite::new(0.5).unwrap(),
            )
            .unwrap();
        assert_eq!(output.alpha(), Finite::new(0.5));
        assert_eq!(output.channels(), [Finite::new(expected), None, None]);
    }
}

#[test]
fn sampled_endpoint_at_exact_progress_keeps_its_original_space_and_missing_components() {
    let source_method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let source = color(SRGB, [Some(-0.0), None, Some(0.5)], None);
    let sample = source
        .sample_interpolation(source, source_method, Finite::new(0.5).unwrap())
        .unwrap();
    let space = ColorSpace::Oklch;
    let target_method =
        InterpolationMethod::try_new(space, Some(HueInterpolation::Longer)).unwrap();
    let target = color(space, [Some(0.5), Some(0.25), Some(180.0)], Some(1.0));
    let sample_endpoint = InterpolationEndpoint::Sampled(sample);
    let resolved_endpoint = InterpolationEndpoint::Resolved(target);

    for (first, second, progress) in [
        (sample_endpoint, resolved_endpoint, -0.0),
        (resolved_endpoint, sample_endpoint, 1.0),
    ] {
        let result = first
            .sample_interpolation(second, target_method, Finite::new(progress).unwrap())
            .unwrap();
        assert_eq!(result.space(), SRGB);
        assert_eq!(result.alpha(), None);
        assert_eq!(
            result.channels(),
            [Finite::new(-0.0), None, Finite::new(0.5)]
        );
        assert_eq!(
            result.channels()[0].unwrap().get().to_bits(),
            (-0.0_f64).to_bits()
        );
    }
}

#[test]
fn sampled_endpoint_carries_missing_alpha_and_channels_into_the_new_pair() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let first = color(SRGB, [Some(0.25), None, None], None);
    let sample = first
        .sample_interpolation(first, method, Finite::new(0.5).unwrap())
        .unwrap();

    for alpha in [None, Some(0.5)] {
        let second = color(SRGB, [Some(0.75), Some(0.25), None], alpha);
        let result = InterpolationEndpoint::Sampled(sample)
            .sample_interpolation(
                InterpolationEndpoint::Resolved(second),
                method,
                Finite::new(0.5).unwrap(),
            )
            .unwrap();
        assert_eq!(
            result.channels(),
            [Finite::new(0.5), Finite::new(0.25), None]
        );
        assert_eq!(result.alpha(), alpha.and_then(Finite::new));
    }
}

#[test]
fn sampled_endpoint_converts_extended_alpha_and_uses_the_new_hue_arc() {
    let source_method = InterpolationMethod::try_new(ColorSpace::Oklab, None).unwrap();
    let channels = [Some(0.5), Some(0.125), Some(0.0)];
    let sample = color(ColorSpace::Oklab, channels, Some(0.0))
        .sample_interpolation(
            color(ColorSpace::Oklab, channels, Some(1.0)),
            source_method,
            Finite::new(2.0).unwrap(),
        )
        .unwrap();
    let space = ColorSpace::Oklch;
    let method = InterpolationMethod::try_new(space, Some(HueInterpolation::Longer)).unwrap();
    let target = color(space, [Some(0.75), Some(0.5), Some(90.0)], Some(1.0));
    let result = InterpolationEndpoint::Sampled(sample)
        .sample_interpolation(
            InterpolationEndpoint::Resolved(target),
            method,
            Finite::new(0.5).unwrap(),
        )
        .unwrap();

    // The conversion gives Oklch (1/2, 1/8, 0) at alpha 2. Weighted
    // lightness is (2*1/2 + 3/4)/3 and chroma is (2*1/8 + 1/2)/3.
    // Hue takes the 270-degree arc from 0 toward 90 independently of alpha.
    assert_eq!(result.space(), space);
    assert_eq!(result.alpha(), Finite::new(1.5));
    assert_eq!(
        result.channels(),
        [
            Finite::new(7.0 / 12.0),
            Finite::new(0.25),
            Finite::new(225.0)
        ]
    );
}

#[test]
fn sampled_endpoint_converts_zero_alpha_residue_and_retains_its_contribution() {
    let source_method = InterpolationMethod::try_new(ColorSpace::Oklab, None).unwrap();
    let first = color(
        ColorSpace::Oklab,
        [Some(0.75), Some(0.25), Some(0.0)],
        Some(0.5),
    );
    let second = color(
        ColorSpace::Oklab,
        [Some(0.5), Some(0.125), Some(0.0)],
        Some(1.0),
    );
    let sample = first
        .sample_interpolation(second, source_method, Finite::new(-1.0).unwrap())
        .unwrap();
    let space = ColorSpace::Oklch;
    let method = InterpolationMethod::try_new(space, None).unwrap();
    let target = color(space, [Some(0.5), Some(0.125), Some(90.0)], Some(1.0));
    let result = InterpolationEndpoint::Sampled(sample)
        .sample_interpolation(
            InterpolationEndpoint::Resolved(target),
            method,
            Finite::new(0.5).unwrap(),
        )
        .unwrap();

    // The Oklab residue (1/4, 1/8, 0) converts to the same Oklch tuple.
    // The half weights and divisor of 1/2 leave the sum of the two
    // non-hue endpoint contributions.
    assert_eq!(result.alpha(), Finite::new(0.5));
    assert_eq!(
        result.channels(),
        [Finite::new(0.75), Finite::new(0.25), Finite::new(45.0)]
    );
}

#[test]
fn converting_a_zero_alpha_sample_without_residue_adds_no_premultiplied_blackness() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let transparent = color(SRGB, [Some(0.0); 3], Some(0.0));
    let sample = transparent
        .sample_interpolation(transparent, method, Finite::new(0.5).unwrap())
        .unwrap();

    let space = ColorSpace::Hwb;
    let method = InterpolationMethod::try_new(space, None).unwrap();
    let target = color(space, [Some(120.0), Some(25.0), Some(50.0)], Some(1.0));
    let result = InterpolationEndpoint::Sampled(sample)
        .sample_interpolation(
            InterpolationEndpoint::Resolved(target),
            method,
            Finite::new(0.5).unwrap(),
        )
        .unwrap();

    // RGB black converts to 100% blackness. Its coordinates are straight
    // and alpha zero multiplies that blackness away before interpolation.
    assert_eq!(result.alpha(), Finite::new(0.5));
    assert_eq!(result.channels(), target.finite_channels());
}

#[test]
fn sampled_endpoint_blends_signed_alpha_through_zero_and_back_to_positive() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let channels = [Some(0.25), None, None];
    let negative = color(SRGB, channels, Some(1.0))
        .sample_interpolation(
            color(SRGB, channels, Some(0.0)),
            method,
            Finite::new(2.0).unwrap(),
        )
        .unwrap();
    let from = InterpolationEndpoint::Sampled(negative);
    let to = InterpolationEndpoint::Resolved(color(SRGB, [Some(0.75), None, None], Some(1.0)));

    // Alpha is -1+2p, and the premultiplied coordinate is -1/4+p.
    // At alpha zero the sample keeps the numerator. At 1/2 it divides.
    for (progress, alpha, coordinate) in [(0.5, 0.0, 0.25), (0.75, 0.5, 1.0)] {
        let result = from
            .sample_interpolation(to, method, Finite::new(progress).unwrap())
            .unwrap();
        assert_eq!(result.alpha(), Finite::new(alpha));
        assert_eq!(result.channels(), [Finite::new(coordinate), None, None]);
    }
}

#[test]
fn sampled_endpoint_rejects_true_channel_overflow_inside_unit_progress() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let channels = [Some(f64::MAX), None, None];
    let negative = color(SRGB, channels, Some(1.0))
        .sample_interpolation(
            color(SRGB, channels, Some(0.0)),
            method,
            Finite::new(2.0).unwrap(),
        )
        .unwrap();
    let target = color(SRGB, [Some(f64::MAX.next_down()), None, None], Some(1.0));

    // The normalized weights are 7/6 and -1/6. The result exceeds MAX
    // by 1/6 ULP even though binary64 rounding would return MAX.
    let error = InterpolationEndpoint::Sampled(negative)
        .sample_interpolation(
            InterpolationEndpoint::Resolved(target),
            method,
            Finite::new(0.125).unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Channel(0));
}

#[test]
fn zero_alpha_residue_uses_separate_channel_and_divisor_weights() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let black = color(SRGB, [Some(0.0), None, None], Some(1.0));
    let sample = color(SRGB, [Some(f64::MAX), None, None], Some(0.5))
        .sample_interpolation(black, method, Finite::new(-1.0).unwrap())
        .unwrap();
    let from = InterpolationEndpoint::Sampled(sample);
    let to = InterpolationEndpoint::Resolved(black);

    // The sample has alpha zero and residue MAX. At progress p, the
    // straight channel is (1 - p) * MAX / p. It is MAX at the midpoint
    // and above the finite bound one representable step earlier.
    let result = from
        .sample_interpolation(to, method, Finite::new(0.5).unwrap())
        .unwrap();
    assert_eq!(result.channels(), [Finite::new(f64::MAX), None, None]);
    let error = from
        .sample_interpolation(to, method, Finite::new(0.5_f64.next_down()).unwrap())
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Channel(0));
}

#[test]
fn sampled_endpoint_checks_range_with_three_large_factors() {
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let maximum = Finite::new(f64::MAX).unwrap();
    let sample = |coordinate| {
        let channels = [Some(coordinate), None, None];
        let value = color(SRGB, channels, Some(0.0))
            .sample_interpolation(color(SRGB, channels, Some(1.0)), method, maximum)
            .unwrap();
        InterpolationEndpoint::Sampled(value)
    };
    let high = 2.0_f64.powi(1023);

    // The shared alpha stays MAX. The channel increases by 2^971 * MAX,
    // so the numerator has three large factors and the result exceeds MAX.
    let error = sample(high)
        .sample_interpolation(sample(high.next_up()), method, maximum)
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Channel(0));

    // Alpha is MAX + MAX * 2^-1074. Its true range fails even though the
    // excess is tiny compared to the spacing around MAX.
    let error = sample(high)
        .sample_interpolation(
            InterpolationEndpoint::Resolved(color(SRGB, [Some(high), None, None], Some(0.0))),
            method,
            Finite::new(-f64::from_bits(1)).unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Alpha);
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

#[test]
fn sample_preserves_a_channel_slope_smaller_than_the_premultiplied_spacing() {
    let first = color(SRGB, [Some(1.0), None, None], Some(1.0));
    let second = color(
        SRGB,
        [Some(1.0_f64.next_up()), None, None],
        Some(1.0_f64.next_down()),
    );
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // The exact second premultiplied endpoint is 1 + 2^-53 - 2^-105.
    // Multiplying that slope by 2^54 and adding 1 gives 3 - 2^-51.
    let sample = first
        .sample_interpolation(second, method, Finite::new(2.0_f64.powi(54)).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(-1.0));
    assert_eq!(
        sample.channels()[0].map(Finite::get),
        Some(-3.0_f64.next_down())
    );
}

#[test]
fn sample_recovers_a_nonconstant_maximum_channel_after_division() {
    let max = f64::MAX;
    let ulp = 2.0_f64.powi(971);
    let first = color(SRGB, [Some(max - 4.0 * ulp), None, None], Some(1.0 / 16.0));
    let second = color(SRGB, [Some(max - 2.0 * ulp), None, None], Some(3.0 / 16.0));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // The exact normalized channel weights at t = -2 are -1 and 2.
    let sample = first
        .sample_interpolation(second, method, Finite::new(-2.0).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(-3.0 / 16.0));
    assert_eq!(sample.channels()[0].map(Finite::get), Some(max));
}

#[test]
fn sample_keeps_the_arc_from_a_tiny_negative_hue() {
    let space = ColorSpace::Oklch;
    let progress = 2.0_f64.powi(1023);
    let first = color(
        space,
        [Some(0.5), Some(0.25), Some(-1.0 / progress)],
        Some(1.0),
    );
    let second = color(space, [Some(0.5), Some(0.25), Some(0.0)], Some(1.0));
    let method = InterpolationMethod::try_new(space, Some(HueInterpolation::Increasing)).unwrap();

    // The increasing arc crosses zero with a positive step of 2^-1023.
    // Advancing by progress adds exactly one degree to the tiny negative hue.
    let sample = first
        .sample_interpolation(second, method, Finite::new(progress).unwrap())
        .unwrap();
    assert_eq!(sample.channels()[2].map(Finite::get), Some(1.0));
}

#[test]
fn ordinary_and_controlled_interpolation_keep_exact_premultiplied_cancellation() {
    let first = color(SRGB, [Some(-2.0_f64.powi(1022)), None, None], Some(1.0));
    let second = color(SRGB, [Some(2.0_f64.powi(1023)), None, None], Some(0.5));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let progress = UnitInterval::new(0.5).unwrap();
    let request = NumericalRequest::Interpolate {
        first,
        second,
        method,
        progress,
    };
    let mut owner = OperationOwner::new();
    let ordinary = first.interpolated(second, method, progress);
    let controlled = Computation::new(&mut owner, request).compute();
    let checked = first
        .sample_interpolation(second, method, Finite::new(0.5).unwrap())
        .unwrap();

    // The premultiplied terms have equal magnitude 2^1021 and opposite signs.
    assert_eq!(ordinary.channels(), [Some(0.0), None, None]);
    assert_eq!(controlled.channels(), [Some(0.0), None, None]);
    assert_eq!(checked.channels()[0].map(Finite::get), Some(0.0));
    assert_eq!(checked.alpha().map(Finite::get), Some(0.75));
}

#[test]
fn sample_keeps_exact_cancellation_during_extrapolation() {
    let first = color(SRGB, [Some(2.0_f64.powi(1023)), None, None], Some(0.5));
    let second = color(SRGB, [Some(2.0_f64.powi(1021)), None, None], Some(1.0));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // At t = 2 the weights are -1/2 and 2. The two terms are equal
    // and opposite, so their sum is zero before dividing by 3/2.
    let sample = first
        .sample_interpolation(second, method, Finite::new(2.0).unwrap())
        .unwrap();
    assert_eq!(sample.alpha().map(Finite::get), Some(1.5));
    assert_eq!(sample.channels()[0].map(Finite::get), Some(0.0));
}

#[test]
fn sample_rejects_true_overflow_at_the_finite_boundary() {
    let first = color(
        SRGB,
        [Some(f64::from_bits(0xffd5_5555_5555_5566)), None, None],
        Some(0.5),
    );
    let second = color(
        SRGB,
        [Some(f64::from_bits(0x7fe5_5555_5555_5553)), None, None],
        Some(1.0),
    );
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();

    // At t = 2 the exact result is MAX plus 2/3 of a top-binade ULP.
    let error = first
        .sample_interpolation(second, method, Finite::new(2.0).unwrap())
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Channel(0));
}

#[test]
fn sample_applies_the_exact_range_boundary_to_alpha() {
    let first = color(SRGB, [None; 3], Some(1.0));
    let second = color(SRGB, [None; 3], Some(0.0));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    // 1 - (-MAX) lies outside the finite range even though it rounds to MAX.
    let error = first
        .sample_interpolation(second, method, Finite::new(-f64::MAX).unwrap())
        .unwrap_err();
    assert_eq!(error.component(), InterpolationComponent::Alpha);

    let valid = second
        .sample_interpolation(first, method, Finite::new(f64::MAX).unwrap())
        .unwrap();
    assert_eq!(valid.alpha().map(Finite::get), Some(f64::MAX));
    assert_eq!(valid.channels(), [None; 3]);
}
