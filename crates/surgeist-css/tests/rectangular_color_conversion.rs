use surgeist_css::{
    CssPredefinedColorSpace as P, CssRectangularColorConversionError as Error,
    CssRectangularColorCoordinates as Coordinates, CssRectangularColorSpace as S,
};

const SPACES: [S; 11] = [
    S::Predefined(P::Srgb),
    S::Predefined(P::SrgbLinear),
    S::Predefined(P::DisplayP3),
    S::Predefined(P::DisplayP3Linear),
    S::Predefined(P::A98Rgb),
    S::Predefined(P::ProphotoRgb),
    S::Predefined(P::Rec2020),
    S::Predefined(P::XyzD50),
    S::Predefined(P::XyzD65),
    S::Lab,
    S::Oklab,
];
const D50: [f64; 3] = [3457.0 / 3585.0, 1.0, 986.0 / 1195.0];
const D65: [f64; 3] = [3127.0 / 3290.0, 1.0, 3583.0 / 3290.0];

fn color(space: S, values: [f64; 3]) -> Coordinates {
    Coordinates::try_new(space, values.map(Some)).unwrap()
}

fn converted(source: S, destination: S, values: [f64; 3]) -> [f64; 3] {
    let result = color(source, values).convert_to(destination).unwrap();
    assert_eq!(result.space(), destination);
    result.channels().map(Option::unwrap)
}

fn close(actual: [f64; 3], expected: [f64; 3], tolerance: f64) {
    for i in 0..3 {
        assert!(actual[i].is_finite());
        assert!(
            (actual[i] - expected[i]).abs() <= tolerance * expected[i].abs().max(1.0),
            "channel {i}: actual {:?}, expected {:?}",
            actual,
            expected
        );
    }
}

fn relative(actual: f64, expected: f64, tolerance: f64) {
    assert!(actual.is_finite());
    assert!(
        (actual / expected - 1.0).abs() <= tolerance,
        "actual {actual}, expected {expected}"
    );
}

// Independent numerical oracle: derive RGB matrices from the normative xy
// primaries rather than importing conversion coefficients. Gaussian solves also
// supply inverse and Bradford cone-domain conversions independently of the
// implementation's precomputed matrices. The direct f64 formulas below evaluate
// the pinned source in ordinary magnitudes; they never use the private Wide code.
type Matrix = [[f64; 3]; 3];

fn solve(matrix: Matrix, rhs: [f64; 3]) -> [f64; 3] {
    let mut a: [[f64; 4]; 3] =
        std::array::from_fn(|i| [matrix[i][0], matrix[i][1], matrix[i][2], rhs[i]]);
    for col in 0..3 {
        let pivot = (col..3)
            .max_by(|&a_row, &b_row| a[a_row][col].abs().total_cmp(&a[b_row][col].abs()))
            .unwrap();
        a.swap(col, pivot);
        let divisor = a[col][col];
        for v in &mut a[col] {
            *v /= divisor;
        }
        for row in 0..3 {
            if row != col {
                let multiplier = a[row][col];
                let pivot_row = a[col];
                for (value, pivot_value) in a[row].iter_mut().zip(pivot_row) {
                    *value -= multiplier * pivot_value;
                }
            }
        }
    }
    [a[0][3], a[1][3], a[2][3]]
}

fn product(matrix: Matrix, vector: [f64; 3]) -> [f64; 3] {
    matrix.map(|row| row[0] * vector[0] + row[1] * vector[1] + row[2] * vector[2])
}

fn primaries(space: P) -> [[f64; 2]; 3] {
    match space {
        P::Srgb | P::SrgbLinear => [[0.64, 0.33], [0.30, 0.60], [0.15, 0.06]],
        P::DisplayP3 | P::DisplayP3Linear => [[0.68, 0.32], [0.265, 0.69], [0.15, 0.06]],
        P::A98Rgb => [[0.64, 0.33], [0.21, 0.71], [0.15, 0.06]],
        P::ProphotoRgb => [
            [0.734699, 0.265301],
            [0.159597, 0.840403],
            [0.036598, 0.000105],
        ],
        P::Rec2020 => [[0.708, 0.292], [0.170, 0.797], [0.131, 0.046]],
        _ => panic!("test requested RGB primaries for {space:?}"),
    }
}

fn white(space: S) -> [f64; 3] {
    match space {
        S::Lab | S::Predefined(P::ProphotoRgb | P::XyzD50) => D50,
        _ => D65,
    }
}

fn rgb_matrix(space: P) -> Matrix {
    let xy = primaries(space);
    let columns = xy.map(|[x, y]| [x / y, 1.0, (1.0 - x - y) / y]);
    let basis = std::array::from_fn(|row| std::array::from_fn(|col| columns[col][row]));
    let weights = solve(basis, white(S::Predefined(space)));
    std::array::from_fn(|row| std::array::from_fn(|col| basis[row][col] * weights[col]))
}

fn adapt(xyz: [f64; 3], from: [f64; 3], to: [f64; 3]) -> [f64; 3] {
    if from == to {
        return xyz;
    }
    // Published Bradford cone basis, not the implementation adaptation matrix.
    let cone = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];
    let src = product(cone, from);
    let dst = product(cone, to);
    let response = product(cone, xyz);
    solve(cone, std::array::from_fn(|i| response[i] * dst[i] / src[i]))
}

fn decode(c: f64, space: P) -> f64 {
    let a = c.abs();
    match space {
        P::Srgb | P::DisplayP3 => {
            if a <= 0.04045 {
                c / 12.92
            } else {
                (((a + 0.055) / 1.055).powf(2.4)).copysign(c)
            }
        }
        P::A98Rgb => a.powf(563.0 / 256.0).copysign(c),
        P::ProphotoRgb => {
            if a <= 1.0 / 32.0 {
                c / 16.0
            } else {
                a.powf(1.8).copysign(c)
            }
        }
        P::Rec2020 => a.powf(2.4).copysign(c),
        _ => c,
    }
}

fn encode(c: f64, space: P) -> f64 {
    let a = c.abs();
    match space {
        P::Srgb | P::DisplayP3 => {
            if a <= 0.0031308 {
                c * 12.92
            } else {
                (1.055 * a.powf(1.0 / 2.4) - 0.055).copysign(c)
            }
        }
        P::A98Rgb => a.powf(256.0 / 563.0).copysign(c),
        P::ProphotoRgb => {
            if a <= 1.0 / 512.0 {
                c * 16.0
            } else {
                a.powf(1.0 / 1.8).copysign(c)
            }
        }
        P::Rec2020 => a.powf(1.0 / 2.4).copysign(c),
        _ => c,
    }
}

fn lab_xyz([l, a, b]: [f64; 3]) -> [f64; 3] {
    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;
    let inverse = |v: f64| {
        if v.powi(3) > 216.0 / 24389.0 {
            v.powi(3)
        } else {
            (116.0 * v - 16.0) * 27.0 / 24389.0
        }
    };
    [
        inverse(fx) * D50[0],
        if l > 8.0 {
            fy.powi(3)
        } else {
            l * 27.0 / 24389.0
        },
        inverse(fz) * D50[2],
    ]
}

fn xyz_lab(xyz: [f64; 3]) -> [f64; 3] {
    let f: [f64; 3] = std::array::from_fn(|i| {
        let t = xyz[i] / D50[i];
        if t > 216.0 / 24389.0 {
            t.cbrt()
        } else {
            (24389.0 / 27.0 * t + 16.0) / 116.0
        }
    });
    [
        116.0 * f[1] - 16.0,
        500.0 * (f[0] - f[1]),
        200.0 * (f[1] - f[2]),
    ]
}

fn oklab_xyz([l, a, b]: [f64; 3]) -> [f64; 3] {
    // Published Color 4 §19 formula, evaluated independently as scalar equations.
    let p = (l + 0.3963377773761749 * a + 0.2158037573099136 * b).powi(3);
    let q = (l - 0.1055613458156586 * a - 0.0638541728258133 * b).powi(3);
    let r = (l - 0.0894841775298119 * a - 1.2914855480194092 * b).powi(3);
    [
        1.2268798758459243 * p - 0.5578149944602171 * q + 0.2813910456659647 * r,
        -0.0405757452148008 * p + 1.112286803280317 * q - 0.0717110580655164 * r,
        -0.0763729366746601 * p - 0.4214933324022432 * q + 1.5869240198367816 * r,
    ]
}

fn xyz_oklab([x, y, z]: [f64; 3]) -> [f64; 3] {
    let p = (0.819022437996703 * x + 0.3619062600528904 * y - 0.1288737815209879 * z).cbrt();
    let q = (0.0329836539323885 * x + 0.9292868615863434 * y + 0.0361446663506424 * z).cbrt();
    let r = (0.0481771893596242 * x + 0.2642395317527308 * y + 0.6335478284694309 * z).cbrt();
    [
        0.210454268309314 * p + 0.7936177747023054 * q - 0.0040720430116193 * r,
        1.9779985324311684 * p - 2.42859224204858 * q + 0.450593709617411 * r,
        0.0259040424655478 * p + 0.7827717124575296 * q - 0.8086757549230774 * r,
    ]
}

fn reference(source: S, destination: S, input: [f64; 3]) -> [f64; 3] {
    if source == destination {
        return input;
    }
    let xyz = match source {
        S::Predefined(P::XyzD50 | P::XyzD65) => input,
        S::Predefined(rgb) => product(rgb_matrix(rgb), input.map(|v| decode(v, rgb))),
        S::Lab => lab_xyz(input),
        S::Oklab => oklab_xyz(input),
        _ => panic!("unhandled test space"),
    };
    let xyz = adapt(xyz, white(source), white(destination));
    match destination {
        S::Predefined(P::XyzD50 | P::XyzD65) => xyz,
        S::Predefined(rgb) => solve(rgb_matrix(rgb), xyz).map(|v| encode(v, rgb)),
        S::Lab => xyz_lab(xyz),
        S::Oklab => xyz_oklab(xyz),
        _ => panic!("unhandled test space"),
    }
}

#[test]
fn all_rectangular_spaces_convert_unequal_normal_and_extended_values_by_independent_formulas() {
    for source in SPACES {
        let cases = match source {
            S::Lab => [[53.0, 17.0, -23.0], [-4.0, 170.0, -230.0]],
            S::Oklab => [[0.6, 0.07, -0.03], [-0.2, 0.7, -0.4]],
            S::Predefined(P::XyzD50 | P::XyzD65) => [[0.23, 0.4, 0.61], [-0.2, 1.1, 0.7]],
            _ => [[0.18, 0.42, 0.7], [-0.05, 1.12, 0.34]],
        };
        for input in cases {
            for destination in SPACES {
                let expected = reference(source, destination, input);
                let actual = converted(source, destination, input);
                close(actual, expected, 2e-11);
            }
        }
    }
}

#[test]
fn each_rgb_primary_and_white_has_the_normative_chromaticity_and_scale() {
    for space in [
        P::Srgb,
        P::SrgbLinear,
        P::DisplayP3,
        P::DisplayP3Linear,
        P::A98Rgb,
        P::ProphotoRgb,
        P::Rec2020,
    ] {
        let src = S::Predefined(space);
        let dest = if space == P::ProphotoRgb {
            S::Predefined(P::XyzD50)
        } else {
            S::Predefined(P::XyzD65)
        };
        close(converted(src, dest, [1.0; 3]), white(src), 2e-15);
        close(converted(dest, src, white(src)), [1.0; 3], 4e-15);
        for i in 0..3 {
            let primary = std::array::from_fn(|j| if i == j { 1.0 } else { 0.0 });
            let xyz = converted(src, dest, primary);
            let sum: f64 = xyz.iter().sum();
            let xy = primaries(space)[i];
            assert!((xyz[0] / sum - xy[0]).abs() < 2e-15);
            assert!((xyz[1] / sum - xy[1]).abs() < 2e-15);
            // Up to eight binary64 epsilons of matrix cancellation near zero
            // becomes <1e-6 after the strongest inverse gamma (1/2.4).
            close(converted(dest, src, xyz), primary, 1e-6);
        }
    }
}

#[test]
fn p3_red_retains_exactly_derived_out_of_gamut_linear_srgb_channels() {
    // Independent exact-rational primary/whitepoint solve, not implementation output.
    close(
        converted(
            S::Predefined(P::DisplayP3),
            S::Predefined(P::SrgbLinear),
            [1.0, 0.0, 0.0],
        ),
        [
            3685649.0 / 3008840.0,
            -5617931.0 / 133579120.0,
            -1323971.0 / 67420360.0,
        ],
        2e-15,
    );
}

#[test]
fn transfer_toes_neighbors_signs_and_nonlinear_controls_follow_the_selected_curves() {
    for (encoded, linear) in [(P::Srgb, P::SrgbLinear), (P::DisplayP3, P::DisplayP3Linear)] {
        for threshold in [0.04045_f64, -0.04045] {
            for c in [
                threshold,
                f64::from_bits(threshold.to_bits() - 1),
                f64::from_bits(threshold.to_bits() + 1),
                -0.5,
                0.5,
                1.5,
                -1.5,
            ] {
                close(
                    converted(S::Predefined(encoded), S::Predefined(linear), [c; 3]),
                    [decode(c, encoded); 3],
                    2e-15,
                );
            }
        }
        for threshold in [0.0031308_f64, -0.0031308] {
            for c in [
                threshold,
                f64::from_bits(threshold.to_bits() - 1),
                f64::from_bits(threshold.to_bits() + 1),
                -0.5,
                0.5,
            ] {
                close(
                    converted(S::Predefined(linear), S::Predefined(encoded), [c; 3]),
                    [encode(c, encoded); 3],
                    2e-15,
                );
            }
        }
    }
    for rgb in [P::A98Rgb, P::ProphotoRgb, P::Rec2020] {
        let src = S::Predefined(rgb);
        let dst = if rgb == P::ProphotoRgb {
            S::Predefined(P::XyzD50)
        } else {
            S::Predefined(P::XyzD65)
        };
        let threshold = if rgb == P::ProphotoRgb {
            1.0 / 32.0
        } else {
            0.5_f64
        };
        for a in [
            threshold,
            f64::from_bits(threshold.to_bits() - 1),
            f64::from_bits(threshold.to_bits() + 1),
            0.5,
            1.25,
        ] {
            for c in [a, -a] {
                close(
                    converted(src, dst, [c; 3]),
                    white(src).map(|w| w * decode(c, rgb)),
                    4e-15,
                );
            }
        }
        let threshold = if rgb == P::ProphotoRgb {
            1.0 / 512.0
        } else {
            0.5_f64
        };
        for a in [
            threshold,
            f64::from_bits(threshold.to_bits() - 1),
            f64::from_bits(threshold.to_bits() + 1),
            0.25,
            1.25,
        ] {
            for c in [a, -a] {
                close(
                    converted(dst, src, white(src).map(|w| w * c)),
                    [encode(c, rgb); 3],
                    4e-15,
                );
            }
        }
    }
    let rec_half = converted(
        S::Predefined(P::Rec2020),
        S::Predefined(P::XyzD65),
        [0.5; 3],
    );
    close(rec_half, D65.map(|w| w * 0.18946457081379978), 2e-15);
    let pro_toe = converted(
        S::Predefined(P::ProphotoRgb),
        S::Predefined(P::XyzD50),
        [-1.0 / 32.0; 3],
    );
    close(pro_toe, D50.map(|w| -w / 512.0), 2e-15);
}

#[test]
fn bradford_adaptation_matches_independent_cone_response_and_whitepoints() {
    for (source, destination, from, to) in [
        (P::XyzD65, P::XyzD50, D65, D50),
        (P::XyzD50, P::XyzD65, D50, D65),
    ] {
        close(
            converted(S::Predefined(source), S::Predefined(destination), from),
            to,
            2e-15,
        );
        for xyz in [[0.31, 0.27, 0.89], [-0.4, 1.3, -0.2], [1.0, 0.0, 0.0]] {
            close(
                converted(S::Predefined(source), S::Predefined(destination), xyz),
                adapt(xyz, from, to),
                2e-15,
            );
        }
    }
}

#[test]
fn lab_neutral_black_threshold_negative_and_overrange_lightness_are_unclamped() {
    let xyz = S::Predefined(P::XyzD50);
    for (l, luminance) in [
        (0.0, 0.0),
        (8.0, 216.0 / 24389.0),
        (50.0, 35937.0 / 195112.0),
        (100.0, 1.0),
        (-1.0, -27.0 / 24389.0),
        (116.0, (132.0_f64 / 116.0).powi(3)),
    ] {
        close(
            converted(S::Lab, xyz, [l, 0.0, 0.0]),
            D50.map(|w| w * luminance),
            2e-15,
        );
        close(
            converted(xyz, S::Lab, D50.map(|w| w * luminance)),
            [l, 0.0, 0.0],
            4e-13,
        );
    }
    for l in [
        f64::from_bits(8.0_f64.to_bits() - 1),
        8.0,
        f64::from_bits(8.0_f64.to_bits() + 1),
    ] {
        close(
            converted(S::Lab, xyz, [l, 5.0, -3.0]),
            lab_xyz([l, 5.0, -3.0]),
            2e-15,
        );
    }
    let adapted_white = converted(S::Predefined(P::XyzD65), S::Lab, [1.0; 3]);
    assert!(adapted_white[0] > 100.0);
    close(adapted_white, xyz_lab(adapt([1.0; 3], D65, D50)), 2e-13);
}

#[test]
fn oklab_black_white_unequal_negative_xyz_and_signed_cube_roots_match_published_basis() {
    let xyz = S::Predefined(P::XyzD65);
    close(converted(xyz, S::Oklab, [0.0; 3]), [0.0; 3], 0.0);
    close(converted(S::Oklab, xyz, [0.0; 3]), [0.0; 3], 0.0);
    close(converted(xyz, S::Oklab, D65), [1.0, 0.0, 0.0], 4e-8);
    close(
        converted(S::Oklab, xyz, [0.5, 0.0, 0.0]),
        D65.map(|v| v / 8.0),
        4e-8,
    );
    for input in [[0.25, 0.4, 0.8], [-0.4, -0.2, -0.7], [0.1, -0.7, 0.4]] {
        let positive = converted(xyz, S::Oklab, input);
        close(positive, xyz_oklab(input), 2e-15);
        close(
            converted(xyz, S::Oklab, input.map(|v| -v)),
            positive.map(|v| -v),
            2e-15,
        );
    }
}

#[test]
fn identity_preserves_every_missing_mask_and_all_finite_input_bits() {
    for space in SPACES {
        for mask in 0..8 {
            for values in [
                [-0.0, f64::from_bits(1), f64::MAX],
                [f64::MIN_POSITIVE, -f64::MAX, 0.0],
            ] {
                let channels = std::array::from_fn(|i| {
                    if mask & (1 << i) == 0 {
                        Some(values[i])
                    } else {
                        None
                    }
                });
                let original = Coordinates::try_new(space, channels).unwrap();
                let identity = original.convert_to(space).unwrap();
                assert_eq!(identity.space(), space);
                assert_eq!(
                    identity.channels().map(|v| v.map(f64::to_bits)),
                    channels.map(|v| v.map(f64::to_bits))
                );
            }
        }
    }
}

#[test]
fn different_space_conversion_substitutes_each_missing_channel_locally_and_leaves_source_unchanged()
{
    for source in SPACES {
        for mask in 0..8 {
            let values = match source {
                S::Lab => [42.0, 11.0, -9.0],
                _ => [0.2, 0.4, 0.6],
            };
            let channels = std::array::from_fn(|i| {
                if mask & (1 << i) == 0 {
                    Some(values[i])
                } else {
                    None
                }
            });
            let original = Coordinates::try_new(source, channels).unwrap();
            for destination in SPACES.into_iter().filter(|&space| space != source) {
                let actual = original.convert_to(destination).unwrap();
                assert!(actual.channels().iter().all(Option::is_some));
                // The independent missing-component rule requires exactly the
                // same numerical input as an explicitly present zero. Compare
                // bits, including rounded zeros, rather than imposing a global
                // precision tolerance on a nonlinear near-zero destination.
                let explicit = color(source, channels.map(|v| v.unwrap_or(0.0)))
                    .convert_to(destination)
                    .unwrap();
                assert_eq!(
                    actual.channels().map(|v| v.map(f64::to_bits)),
                    explicit.channels().map(|v| v.map(f64::to_bits))
                );
                assert_eq!(original.channels(), &channels);
            }
        }
    }
}

#[test]
fn algebraically_zero_a98_green_obeys_a_propagated_rounding_bound_without_a_gamma_precision_claim()
{
    // Independent exact rational primary derivation gives A98 inverse-green *
    // sRGB-to-XYZ = [0,1,0]. Thus source G=0 has exact destination linear G=0.
    // A conservative 32u accounts for coefficient rounding and the two matrix
    // dot products; |encoded G| <= linear_bound^(256/563). This is the actual
    // zero-point Hölder bound, where a derivative-based bound is unbounded.
    let input = [0.2, 0.0, 0.6];
    let linear = input.map(|v| decode(v, P::Srgb));
    let matrix = rgb_matrix(P::Srgb);
    let inverse_green: [f64; 3] = [
        -851781.0 / 878810.0,
        1648619.0 / 878810.0,
        36519.0 / 878810.0,
    ];
    let condition: f64 = (0..3)
        .map(|j| {
            inverse_green[j].abs()
                * (0..3)
                    .map(|k| matrix[j][k].abs() * linear[k].abs())
                    .sum::<f64>()
        })
        .sum();
    let linear_bound = 32.0 * (f64::EPSILON / 2.0) * condition;
    let actual = converted(S::Predefined(P::Srgb), S::Predefined(P::A98Rgb), input);
    assert!(actual[1].abs() <= linear_bound.powf(256.0 / 563.0));
    assert!(decode(actual[1], P::A98Rgb).abs() <= linear_bound);
    // Nonzero red/blue still receive independently derived direct-formula checks.
    let expected = reference(S::Predefined(P::Srgb), S::Predefined(P::A98Rgb), input);
    assert!((actual[0] - expected[0]).abs() < 2e-15);
    assert!((actual[2] - expected[2]).abs() < 2e-15);
}

#[test]
fn construction_rejects_the_first_present_nonfinite_input_in_channel_order() {
    for space in SPACES {
        for index in 0..3 {
            for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut channels = [None; 3];
                channels[index] = Some(invalid);
                for value in channels.iter_mut().skip(index + 1) {
                    *value = Some(f64::NAN);
                }
                assert_eq!(
                    Coordinates::try_new(space, channels),
                    Err(Error::NonFiniteInput { component: index })
                );
            }
        }
    }
    let error = Error::NonFiniteInput { component: 2 };
    assert!(error.to_string().contains("component 2"));
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn large_encoded_values_survive_linear_overflow_when_the_destination_is_encoded() {
    // Neutral RGB is white-scaled in either primary basis. Both curves have
    // asymptotic gamma 2.4, so Rec2020(c)/c -> 1/1.055. Half-MAX gives explicit
    // headroom for coefficient rounding; direct gamma decoding overflows f64.
    let c = f64::MAX / 2.0;
    for sign in [1.0, -1.0] {
        let source = color(S::Predefined(P::Srgb), [sign * c; 3]);
        let encoded = source.convert_to(S::Predefined(P::Rec2020)).unwrap();
        for channel in encoded.channels() {
            relative(channel.unwrap(), sign * c / 1.055, 3e-14);
        }
        assert_eq!(
            source.convert_to(S::Predefined(P::SrgbLinear)),
            Err(Error::UnrepresentableResult { component: 0 })
        );
        assert_eq!(source.channels(), &[Some(sign * c); 3]);
    }
}

#[test]
fn maximum_xyz_survives_lms_overflow_before_oklab_cube_root_compression() {
    // Homogeneity supplies an independent finite oracle: OK(k*XYZ)=cbrt(k)*OK(XYZ).
    let scale = f64::MAX.cbrt();
    let expected = xyz_oklab([1.0; 3]).map(|v| v * scale);
    let actual = converted(S::Predefined(P::XyzD65), S::Oklab, [f64::MAX; 3]);
    for i in 0..3 {
        relative(actual[i], expected[i], 2e-13);
    }
    close(
        converted(S::Predefined(P::XyzD65), S::Oklab, [-f64::MAX; 3]),
        expected.map(|v| -v),
        2e-13,
    );
}

#[test]
fn subnormal_neutral_rgb_survives_intermediate_underflow_and_final_underflow_is_permitted() {
    let tiny = f64::from_bits(1);
    for sign in [1.0, -1.0] {
        let result = converted(
            S::Predefined(P::Srgb),
            S::Predefined(P::DisplayP3),
            [sign * tiny; 3],
        );
        assert_eq!(result.map(f64::to_bits), [sign * tiny; 3].map(f64::to_bits));
        let linear = converted(
            S::Predefined(P::Srgb),
            S::Predefined(P::SrgbLinear),
            [sign * tiny; 3],
        );
        assert_eq!(
            linear.map(f64::to_bits),
            [0.0_f64.copysign(sign); 3].map(f64::to_bits)
        );
    }
}

#[test]
fn result_overflow_reports_the_first_final_channel_and_preserves_the_source() {
    for index in 0..3 {
        let values = std::array::from_fn(|i| if i < index { 0.0 } else { f64::MAX });
        let source = color(S::Predefined(P::Srgb), values);
        assert_eq!(
            source.convert_to(S::Predefined(P::SrgbLinear)),
            Err(Error::UnrepresentableResult { component: index })
        );
        assert_eq!(source.channels(), &values.map(Some));
    }
    let source = color(S::Oklab, [f64::MAX, 0.0, 0.0]);
    assert_eq!(
        source.convert_to(S::Predefined(P::XyzD65)),
        Err(Error::UnrepresentableResult { component: 0 })
    );
    let error = Error::UnrepresentableResult { component: 1 };
    assert!(error.to_string().contains("component 1"));
}

#[test]
fn ordinary_round_trips_are_secondary_precision_controls() {
    for source in SPACES {
        let input = match source {
            S::Lab => [48.0, 20.0, -13.0],
            S::Oklab => [0.6, 0.08, -0.04],
            _ => [0.25, 0.45, 0.7],
        };
        for destination in SPACES {
            let restored = converted(destination, source, converted(source, destination, input));
            close(restored, input, 2e-10);
        }
    }
}
