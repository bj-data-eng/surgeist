//! RGB coefficients derive from Color 4 §§10.2–10.8's decimal xy primaries:
//! P has columns [x/y, 1, (1-x-y)/y], S = inverse(P) * white, M = P * diag(S).
//! Rational entries below are exact primary/whitepoint derivations rounded by
//! binary64 division; ProPhoto's largest fractions use rounded decimal values.
//! Bradford is derived from the published four-decimal cone basis and these
//! same whites. Oklab's coefficients are the pinned §19 XYZ/Oklab sample basis,
//! also present in frozen WebKit ColorConversion.cpp:286–345. §19 is informative;
//! it supplies coefficients consistent with the normative conversion routes.

use super::wide::Wide;
use crate::CssPredefinedColorSpace;

type Matrix = [[f64; 3]; 3];

pub(super) const D50: [f64; 3] = [3457.0 / 3585.0, 1.0, 986.0 / 1195.0];

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum WhitePoint {
    D50,
    D65,
}

pub(super) fn apply(matrix: &Matrix, channels: [Wide; 3]) -> [Wide; 3] {
    matrix.map(|row| {
        let mut terms = std::array::from_fn::<_, 3, _>(|i| channels[i].multiply_fixed(row[i]));
        terms.sort_by(|a, b| b.magnitude_order(*a));
        terms[0].add(terms[1]).add(terms[2])
    })
}

pub(super) fn adapt(channels: [Wide; 3], source: WhitePoint, destination: WhitePoint) -> [Wide; 3] {
    if source == destination {
        channels
    } else {
        apply(
            if source == WhitePoint::D50 {
                &D50_TO_D65
            } else {
                &D65_TO_D50
            },
            channels,
        )
    }
}

pub(super) fn rgb_to_xyz(channels: [Wide; 3], space: CssPredefinedColorSpace) -> [Wide; 3] {
    apply(rgb_matrix(space, false), channels)
}

pub(super) fn xyz_to_rgb(channels: [Wide; 3], space: CssPredefinedColorSpace) -> [Wide; 3] {
    apply(rgb_matrix(space, true), channels)
}

fn rgb_matrix(space: CssPredefinedColorSpace, inverse: bool) -> &'static Matrix {
    use CssPredefinedColorSpace::*;
    match (space, inverse) {
        (Srgb | SrgbLinear, false) => &SRGB_TO_XYZ,
        (Srgb | SrgbLinear, true) => &SRGB_FROM_XYZ,
        (DisplayP3 | DisplayP3Linear, false) => &P3_TO_XYZ,
        (DisplayP3 | DisplayP3Linear, true) => &P3_FROM_XYZ,
        (A98Rgb, false) => &A98_TO_XYZ,
        (A98Rgb, true) => &A98_FROM_XYZ,
        (ProphotoRgb, false) => &PROPHOTO_TO_XYZ,
        (ProphotoRgb, true) => &PROPHOTO_FROM_XYZ,
        (Rec2020, false) => &REC2020_TO_XYZ,
        (Rec2020, true) => &REC2020_FROM_XYZ,
        // XYZ is already rectangular linear light and never takes this route.
        (XyzD50 | XyzD65, _) => &IDENTITY,
    }
}

pub(super) fn lab_to_xyz(lab: [Wide; 3]) -> [Wide; 3] {
    let fy = lab[0].add_fixed(16.0).divide_fixed(116.0);
    let fx = lab[1].divide_fixed(500.0).add(fy);
    let fz = fy.add(lab[2].divide_fixed(200.0).negated());
    let inverse = |f: Wide| {
        let cube = f.signed_power(3, 1);
        if cube.greater_than(216.0 / 24389.0) {
            cube
        } else {
            f.multiply_fixed(116.0)
                .add_fixed(-16.0)
                .divide_fixed(24389.0 / 27.0)
        }
    };
    let y = if lab[0].greater_than(8.0) {
        fy.signed_power(3, 1)
    } else {
        lab[0].divide_fixed(24389.0 / 27.0)
    };
    [
        inverse(fx).multiply_fixed(D50[0]),
        y,
        inverse(fz).multiply_fixed(D50[2]),
    ]
}

pub(super) fn xyz_to_lab(xyz: [Wide; 3]) -> [Wide; 3] {
    let f = std::array::from_fn::<_, 3, _>(|i| {
        let value = xyz[i].divide_fixed(D50[i]);
        if value.greater_than(216.0 / 24389.0) {
            value.signed_power(1, 3)
        } else {
            value
                .multiply_fixed(24389.0 / 27.0)
                .add_fixed(16.0)
                .divide_fixed(116.0)
        }
    });
    [
        f[1].multiply_fixed(116.0).add_fixed(-16.0),
        f[0].add(f[1].negated()).multiply_fixed(500.0),
        f[1].add(f[2].negated()).multiply_fixed(200.0),
    ]
}

pub(super) fn oklab_to_xyz(oklab: [Wide; 3]) -> [Wide; 3] {
    apply(
        &LMS_TO_XYZ,
        apply(&OKLAB_TO_LMS, oklab).map(|v| v.signed_power(3, 1)),
    )
}

pub(super) fn xyz_to_oklab(xyz: [Wide; 3]) -> [Wide; 3] {
    apply(
        &LMS_TO_OKLAB,
        apply(&XYZ_TO_LMS, xyz).map(|v| v.signed_power(1, 3)),
    )
}

const IDENTITY: Matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
const SRGB_TO_XYZ: Matrix = [
    [506752.0 / 1228815.0, 87881.0 / 245763.0, 12673.0 / 70218.0],
    [87098.0 / 409605.0, 175762.0 / 245763.0, 12673.0 / 175545.0],
    [7918.0 / 409605.0, 87881.0 / 737289.0, 1001167.0 / 1053270.0],
];
const SRGB_FROM_XYZ: Matrix = [
    [12831.0 / 3959.0, -329.0 / 214.0, -1974.0 / 3959.0],
    [
        -851781.0 / 878810.0,
        1648619.0 / 878810.0,
        36519.0 / 878810.0,
    ],
    [705.0 / 12673.0, -2585.0 / 12673.0, 705.0 / 667.0],
];
const P3_TO_XYZ: Matrix = [
    [
        608311.0 / 1250200.0,
        189793.0 / 714400.0,
        198249.0 / 1000160.0,
    ],
    [
        35783.0 / 156275.0,
        247089.0 / 357200.0,
        198249.0 / 2500400.0,
    ],
    [0.0, 32229.0 / 714400.0, 5220557.0 / 5000800.0],
];
const P3_FROM_XYZ: Matrix = [
    [
        446124.0 / 178915.0,
        -333277.0 / 357830.0,
        -72051.0 / 178915.0,
    ],
    [-14852.0 / 17905.0, 63121.0 / 35810.0, 423.0 / 17905.0],
    [11844.0 / 330415.0, -50337.0 / 660830.0, 316169.0 / 330415.0],
];
const A98_TO_XYZ: Matrix = [
    [
        573536.0 / 994567.0,
        263643.0 / 1420810.0,
        187206.0 / 994567.0,
    ],
    [
        591459.0 / 1989134.0,
        6239551.0 / 9945670.0,
        374412.0 / 4972835.0,
    ],
    [
        53769.0 / 1989134.0,
        351524.0 / 4972835.0,
        4929758.0 / 4972835.0,
    ],
];
const A98_FROM_XYZ: Matrix = [
    [
        1829569.0 / 896150.0,
        -506331.0 / 896150.0,
        -308931.0 / 896150.0,
    ],
    [
        -851781.0 / 878810.0,
        1648619.0 / 878810.0,
        36519.0 / 878810.0,
    ],
    [
        16779.0 / 1248040.0,
        -147721.0 / 1248040.0,
        1266979.0 / 1248040.0,
    ],
];
const PROPHOTO_TO_XYZ: Matrix = [
    [
        0.7977666449006423,
        994367890260633.0 / 7355809637737000.0,
        36085628.0 / 1151139915.0,
    ],
    [0.2880748288194013, 0.711835234241873, 6902.0 / 76742661.0],
    [0.0, 0.0, 986.0 / 1195.0],
];
const PROPHOTO_FROM_XYZ: Matrix = [
    [
        193484287597149.0 / 143770377194000.0,
        -36743695402851.0 / 143770377194000.0,
        -7346934402851.0 / 143770377194000.0,
    ],
    [
        -61079714117883.0 / 112148862602000.0,
        24164038411731.0 / 16021266086000.0,
        2302129882117.0 / 112148862602000.0,
    ],
    [0.0, 0.0, 1195.0 / 986.0],
];
const REC2020_TO_XYZ: Matrix = [
    [
        63426534.0 / 99577255.0,
        20160776.0 / 139408157.0,
        47086771.0 / 278816314.0,
    ],
    [
        26158966.0 / 99577255.0,
        472592308.0 / 697040785.0,
        8267143.0 / 139408157.0,
    ],
    [0.0, 19567812.0 / 697040785.0, 295819943.0 / 278816314.0],
];
const REC2020_FROM_XYZ: Matrix = [
    [
        30757411.0 / 17917100.0,
        -6372589.0 / 17917100.0,
        -4539589.0 / 17917100.0,
    ],
    [
        -19765991.0 / 29648200.0,
        47925759.0 / 29648200.0,
        467509.0 / 29648200.0,
    ],
    [
        792561.0 / 44930125.0,
        -1921689.0 / 44930125.0,
        42328811.0 / 44930125.0,
    ],
];
const D65_TO_D50: Matrix = [
    [
        1.0479297925449966,
        0.022946870601609527,
        -0.050192266289205194,
    ],
    [0.029627808770055674, 0.99043442675388, -0.01707379906341879],
    [
        -0.009243040646204521,
        0.015055191490298164,
        0.751874281428137,
    ],
];
const D50_TO_D65: Matrix = [
    [
        0.9554734214880752,
        -0.023098454948764523,
        0.06325924320057066,
    ],
    [
        -0.028369709333863583,
        1.0099953980813041,
        0.021041441191917306,
    ],
    [
        0.012314014864481996,
        -0.02050764929889898,
        1.330365926242124,
    ],
];
const XYZ_TO_LMS: Matrix = [
    [0.819022437996703, 0.3619062600528904, -0.1288737815209879],
    [0.0329836539323885, 0.9292868615863434, 0.0361446663506424],
    [0.0481771893596242, 0.2642395317527308, 0.6335478284694309],
];
const LMS_TO_OKLAB: Matrix = [
    [0.210454268309314, 0.7936177747023054, -0.0040720430116193],
    [1.9779985324311684, -2.42859224204858, 0.450593709617411],
    [0.0259040424655478, 0.7827717124575296, -0.8086757549230774],
];
const OKLAB_TO_LMS: Matrix = [
    [1.0, 0.3963377773761749, 0.2158037573099136],
    [1.0, -0.1055613458156586, -0.0638541728258133],
    [1.0, -0.0894841775298119, -1.2914855480194092],
];
const LMS_TO_XYZ: Matrix = [
    [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
    [-0.0405757452148008, 1.112286803280317, -0.0717110580655164],
    [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
];

#[cfg(test)]
mod tests {
    use super::{Wide, apply};

    #[test]
    fn large_opposed_matrix_terms_cancel_before_a_small_channel_is_added() {
        // The coefficients are an independent exact algebraic control. Every
        // product of MAX by 2 overflows ordinary binary64, but the sum is tiny.
        let matrix = [[2.0, -2.0, 1.0], [0.0, 0.0, 1.0], [1.0, -1.0, 0.0]];
        let tiny = f64::from_bits(1);
        let result = apply(&matrix, [f64::MAX, f64::MAX, tiny].map(Wide::from_f64));
        assert_eq!(result.map(|v| v.to_f64().unwrap()), [tiny, tiny, 0.0]);
    }
}
