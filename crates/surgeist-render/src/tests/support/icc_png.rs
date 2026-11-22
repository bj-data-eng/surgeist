//! Synthetic profiles authored from ICC.1:2022 §§7.2–7.3, 10 and Annex G.
//! <https://www.color.org/specifications/ICC.1-2022-05.pdf>

use std::borrow::Cow;

// Display-P3 primaries in signed 15.16 XYZ, already Bradford-adapted from D65
// to the ICC D50 PCS. The inverse adaptation belongs to the destination.
// <https://www.w3.org/TR/css-color-4/#color-conversion-code>
const RED: [i32; 3] = [33759, 15807, -69];
const GREEN: [i32; 3] = [19135, 45367, 2745];
const BLUE: [i32; 3] = [10296, 4363, 51385];
const D50: [i32; 3] = [63190, 65536, 54061];
const D65_TO_D50: [i32; 9] = [68674, 1502, -3291, 1939, 64912, -1119, -606, 988, 49262];

pub(crate) fn matrix_profile(sampled_blue: bool) -> Vec<u8> {
    let blue_curve = if sampled_blue {
        sampled_curve()
    } else {
        // Parametric function zero, x^1.5.
        b"para\0\0\0\0\0\0\0\0\0\x01\x80\0".to_vec()
    };
    v4_profile(
        b"RGB ",
        "Surgeist RGB fixture",
        matrix_tags([RED, GREEN, BLUE], blue_curve),
    )
}

pub(crate) fn piecewise_p3_profile(with_mpe: bool) -> Vec<u8> {
    // ICC function type 3 uses the piecewise sRGB transfer used by Display P3.
    // g, a, b, c, d are independently rounded to signed 15.16.
    let mut transfer = fixed(b"para", &[157286, 62119, 3417, 5072, 2651]);
    transfer.splice(8..8, [0, 3, 0, 0]);
    let mut tags = vec![
        (*b"rXYZ", fixed(b"XYZ ", &RED)),
        (*b"gXYZ", fixed(b"XYZ ", &GREEN)),
        (*b"bXYZ", fixed(b"XYZ ", &BLUE)),
        (*b"rTRC", transfer.clone()),
        (*b"gTRC", transfer.clone()),
        (*b"bTRC", transfer),
    ];
    if with_mpe {
        tags.push((*b"D2B1", unsupported_mpe_transform()));
    }
    v4_profile(b"RGB ", "Surgeist piecewise P3 fixture", tags)
}

pub(crate) fn mpe_matrix_profile() -> Vec<u8> {
    // Convert linear Display-P3 to D50 PCSXYZ. The A2B routes are
    // deliberately different so precedence is visible in the decoded pixels.
    let coefficients: Vec<_> = (0..3)
        .flat_map(|row| [RED, GREEN, BLUE].map(|column| column[row] as f32 / 65536.0))
        .collect();
    let matrix = mpe_matrix(3, 3, &coefficients, &[0.0; 3]);
    mpe_profile(
        mpe_transform([matrix]),
        b"XYZ ",
        lut16_transform(),
        Some(lut_a_to_b_transform()),
    )
}

pub(crate) fn mpe_profile(
    transform: Vec<u8>,
    pcs: &[u8; 4],
    perceptual: Vec<u8>,
    colorimetric: Option<Vec<u8>>,
) -> Vec<u8> {
    let mut tags = vec![(*b"D2B1", transform), (*b"A2B0", perceptual)];
    if let Some(transform) = colorimetric {
        tags.push((*b"A2B1", transform));
    }
    let mut profile = v4_profile(b"RGB ", "Surgeist MPE fixture", tags);
    profile[20..24].copy_from_slice(pcs);
    profile
}

pub(crate) fn unsupported_mpe_transform() -> Vec<u8> {
    mpe_transform([mpe_element(b"xmpl", 3, 3)])
}

pub(crate) fn mpe_aliased_matrix_transform() -> Vec<u8> {
    let d50 = [0.9642_f32, 1.0, 0.8249];
    let tone = mpe_matrix(
        3,
        3,
        &[d50[0], 0.0, 0.0, d50[1], 0.0, 0.0, d50[2], 0.0, 0.0],
        &d50.map(|component| component / 4.0),
    );
    let half = mpe_matrix(
        3,
        3,
        &[0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5],
        &[0.0; 3],
    );
    // ICC.1:2022 §10.16 permits exact aliases. Store tone then half, with
    // three positions specifying half, tone, and the same half again.
    let mut transform = mpe_element(b"mpet", 3, 3);
    transform.extend(3_u32.to_be_bytes());
    let tone_start = 16_u32 + 3 * 8;
    let tone_size = u32::try_from(tone.len()).unwrap();
    let tone_position = (tone_start, tone_size);
    let half_position = (tone_start + tone_size, u32::try_from(half.len()).unwrap());
    for (offset, length) in [half_position, tone_position, half_position] {
        transform.extend(offset.to_be_bytes());
        transform.extend(length.to_be_bytes());
    }
    transform.extend(tone);
    transform.extend(half);
    transform
}

pub(crate) fn mpe_overflowing_matrix_transform() -> Vec<u8> {
    let scale_red = |scale| {
        mpe_matrix(
            3,
            3,
            &[scale, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            &[0.0; 3],
        )
    };
    mpe_transform([scale_red(f32::MAX), scale_red(2.0)])
}

pub(crate) fn mpe_clut_transform() -> Vec<u8> {
    let input = mpe_matrix(3, 2, &[2.0, 0.0, 0.0, 0.0, 2.0, 0.0], &[-0.5; 2]);
    let mut begin = mpe_element(b"bACS", 2, 2);
    begin.extend_from_slice(b"test");
    let mut end = mpe_element(b"eACS", 3, 3);
    end.extend_from_slice(b"test");

    // ICC.1:2022 Table 63. Two red positions and three green positions,
    // with green varying fastest and a red/green cross term.
    let mut clut = mpe_element(b"clut", 2, 3);
    clut.extend([2, 3]);
    clut.resize(28, 0);
    for u in [0.0, 1.0] {
        for (v, w) in [(0.0, 0.0), (0.5, 0.25), (1.0, 1.0)] {
            append_floats(
                &mut clut,
                &[
                    -0.125 + u / 8.0 + w / 8.0 + u * w / 4.0,
                    1.125 + v / 8.0 + u / 16.0,
                    0.3125 + u / 8.0 + v / 16.0,
                ],
            );
        }
    }
    let output = mpe_matrix(
        3,
        3,
        &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        &[0.375, -0.75, 0.0],
    );
    mpe_transform([input, begin, clut, end, output])
}

pub(crate) fn mpe_curve_transform() -> Vec<u8> {
    let input = mpe_matrix(
        3,
        3,
        &[1.25, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        &[0.0; 3],
    );
    let mut sampled = b"samf\0\0\0\0".to_vec();
    sampled.extend(2_u32.to_be_bytes());
    append_floats(&mut sampled, &[50.0, 75.0]);
    let lightness = mpe_curve(
        &[0.0, 1.0],
        [constant_segment(20.0), sampled, constant_segment(90.0)],
    );
    let a = mpe_curve(
        &[0.0, 1.0],
        [
            constant_segment(-20.0),
            formula_segment(1, &[2.0, 40.0, 0.9, 0.1, 20.0]),
            constant_segment(20.0),
        ],
    );
    let b = mpe_curve(
        &[0.0],
        [
            constant_segment(14.0),
            formula_segment(2, &[24.0, 2.0, -1.0, 0.0, -10.0]),
        ],
    );
    let mut curves = mpe_element(b"cvst", 3, 3);
    append_positions(&mut curves, [lightness, a, b]);
    mpe_transform([input, curves])
}

pub(crate) fn mpe_formula_transform(
    operand: f32,
    breakpoints: [f32; 2],
    function: u16,
    parameters: &[f32],
) -> Vec<u8> {
    let input = mpe_matrix(3, 1, &[0.0; 3], &[operand]);
    let curve = mpe_curve(
        &breakpoints,
        [
            constant_segment(0.0),
            formula_segment(function, parameters),
            constant_segment(0.0),
        ],
    );
    let mut curves = mpe_element(b"cvst", 1, 1);
    append_positions(&mut curves, [curve]);
    let neutral = mpe_matrix(1, 3, &[0.9642, 1.0, 0.8249], &[0.0; 3]);
    mpe_transform([input, curves, neutral])
}

fn mpe_transform<const N: usize>(elements: [Vec<u8>; N]) -> Vec<u8> {
    let mut transform = mpe_element(b"mpet", 3, 3);
    transform.extend(u32::try_from(N).unwrap().to_be_bytes());
    append_positions(&mut transform, elements);
    transform
}

fn mpe_element(signature: &[u8; 4], input: u16, output: u16) -> Vec<u8> {
    let mut element = signature.to_vec();
    element.extend([0; 4]);
    element.extend(input.to_be_bytes());
    element.extend(output.to_be_bytes());
    element
}

fn mpe_matrix(input: u16, output: u16, coefficients: &[f32], offsets: &[f32]) -> Vec<u8> {
    let mut matrix = mpe_element(b"matf", input, output);
    append_floats(&mut matrix, coefficients);
    append_floats(&mut matrix, offsets);
    matrix
}

fn mpe_curve<const N: usize>(breaks: &[f32], segments: [Vec<u8>; N]) -> Vec<u8> {
    let mut curve = b"curf\0\0\0\0".to_vec();
    curve.extend(u16::try_from(N).unwrap().to_be_bytes());
    curve.extend([0; 2]);
    append_floats(&mut curve, breaks);
    curve.extend(segments.into_iter().flatten());
    curve
}

fn constant_segment(value: f32) -> Vec<u8> {
    formula_segment(0, &[1.0, 0.0, 0.0, value])
}

fn formula_segment(function: u16, parameters: &[f32]) -> Vec<u8> {
    let mut formula = b"parf\0\0\0\0".to_vec();
    formula.extend(function.to_be_bytes());
    formula.extend([0; 2]);
    append_floats(&mut formula, parameters);
    formula
}

fn append_floats(bytes: &mut Vec<u8>, values: &[f32]) {
    bytes.extend(values.iter().flat_map(|value| value.to_be_bytes()));
}

fn append_positions<const N: usize>(bytes: &mut Vec<u8>, values: [Vec<u8>; N]) {
    let table = bytes.len();
    bytes.resize(table + N * 8, 0);
    for (i, value) in values.into_iter().enumerate() {
        let entry = table + i * 8;
        let offset = u32::try_from(bytes.len()).unwrap();
        let length = u32::try_from(value.len()).unwrap();
        bytes[entry..entry + 4].copy_from_slice(&offset.to_be_bytes());
        bytes[entry + 4..entry + 8].copy_from_slice(&length.to_be_bytes());
        bytes.extend(value);
        bytes.resize(bytes.len().next_multiple_of(4), 0);
    }
}

pub(crate) fn v2_matrix_profile() -> Vec<u8> {
    // The v2 XYZ type requires nonnegative colorants. Use the sRGB primaries
    // from CSS Color 4, here Bradford-adapted to the ICC D50 PCS in s15Fixed16.
    let columns = [
        [28576, 14581, 912],
        [25239, 46983, 6361],
        [9375, 3972, 46787],
    ];
    v2_rgb_profile(matrix_tags(columns, sampled_curve()))
}

pub(crate) fn gray_profile() -> Vec<u8> {
    // ICC.1:2022 Table G.3 defines kTRC and the shared input-profile tags.
    v4_profile(
        b"GRAY",
        "Surgeist monochrome fixture",
        vec![(*b"kTRC", curve(&[0x200]))],
    )
}

fn matrix_tags(columns: [[i32; 3]; 3], blue_curve: Vec<u8>) -> Vec<([u8; 4], Vec<u8>)> {
    let [red, green, blue] = columns;
    vec![
        (*b"rXYZ", fixed(b"XYZ ", &red)),
        (*b"gXYZ", fixed(b"XYZ ", &green)),
        (*b"bXYZ", fixed(b"XYZ ", &blue)),
        (*b"rTRC", curve(&[])),      // identity
        (*b"gTRC", curve(&[0x200])), // x² in u8Fixed8
        (*b"bTRC", blue_curve),
    ]
}

fn sampled_curve() -> Vec<u8> {
    // Three samples at x = 0, 1/2, 1, with linear interpolation.
    curve(&[0, 16384, 65535])
}

fn curve(samples: &[u16]) -> Vec<u8> {
    let mut bytes = b"curv\0\0\0\0".to_vec();
    bytes.extend(u32::try_from(samples.len()).unwrap().to_be_bytes());
    bytes.extend(samples.iter().flat_map(|value| value.to_be_bytes()));
    bytes
}

pub(crate) fn lut_profile() -> Vec<u8> {
    lut_profile_with_transform(lut16_transform(), b"XYZ ")
}

pub(crate) fn lut16_transform() -> Vec<u8> {
    // ICC.1:2022 §10.10. A 2×2×2 CLUT transforms RGB to PCSXYZ,
    // bracketed by per-channel identity input and output tables.
    let mut transform = fixed(b"mft2", &[0x10000, 0, 0, 0, 0x10000, 0, 0, 0, 0x10000]);
    transform.splice(8..8, [3, 3, 2, 0]);
    transform.extend_from_slice(&[0, 2, 0, 2]);
    let identity = [0, 0, 255, 255].repeat(3);
    transform.extend_from_slice(&identity);
    for r in [0, 1] {
        for g in [0, 1] {
            for b in [0, 1] {
                for axis in 0..3 {
                    // PCSXYZ u1Fixed15 from the fixture's signed 15.16 XYZ.
                    let value = (r * RED[axis] + g * GREEN[axis] + b * BLUE[axis]).max(0);
                    transform.extend(u16::try_from((value + 1) / 2).unwrap().to_be_bytes());
                }
            }
        }
    }
    transform.extend_from_slice(&identity);
    transform
}

pub(crate) fn lut8_transform() -> Vec<u8> {
    // ICC.1:2022 §10.11. Three 256-entry identity input and output tables
    // surround a 2×2×2 RGB-to-Lab CLUT with neutral chroma and luminance
    // proportional to the three input components.
    let mut transform = fixed(b"mft1", &[0x10000, 0, 0, 0, 0x10000, 0, 0, 0, 0x10000]);
    transform.splice(8..8, [3, 3, 2, 0]);
    let identity: Vec<_> = (0..=u8::MAX).collect();
    transform.extend_from_slice(&identity.repeat(3));
    for lightness in [0, 85, 85, 170, 85, 170, 170, 255] {
        transform.extend_from_slice(&[lightness, 128, 128]);
    }
    transform.extend_from_slice(&identity.repeat(3));
    transform
}

pub(crate) fn lut16_lab_transform() -> Vec<u8> {
    // ICC.1:2022 §10.10. An input table reshapes red before the CLUT.
    // The lightness output table scales its CLUT coordinate to legacy Lab.
    let mut transform = fixed(b"mft2", &[0x10000, 0, 0, 0, 0x10000, 0, 0, 0, 0x10000]);
    transform.splice(8..8, [3, 3, 2, 0]);
    transform.extend_from_slice(&[0, 3, 0, 2]);
    let input: [u16; 9] = [0, 16384, 65535, 0, 32768, 65535, 0, 32768, 65535];
    transform.extend(input.into_iter().flat_map(u16::to_be_bytes));
    for r in [0, 1] {
        for g in [0, 1] {
            for b in [0, 1] {
                let lab = [65535_u16 * r, 0x8000 + 0x2800 * g, 0x8000 - 0x2000 * b];
                transform.extend(lab.into_iter().flat_map(u16::to_be_bytes));
            }
        }
    }
    let output: [u16; 6] = [0, 0xff00, 0, 65535, 0, 65535];
    transform.extend(output.into_iter().flat_map(u16::to_be_bytes));
    transform
}

pub(crate) fn lut_a_to_b_transform() -> Vec<u8> {
    // ICC.1:2022 §10.12 permits a B-curve-only pipeline. With three
    // input and output channels, all three B curves follow the 32-byte
    // prefix and no other process elements are present.
    let mut transform = b"mAB \0\0\0\0\x03\x03\0\0".to_vec();
    transform.extend(32_u32.to_be_bytes());
    transform.extend_from_slice(&[0; 16]);
    for _ in 0..3 {
        transform.extend(curve(&[]));
    }
    transform
}

pub(crate) enum LabStages {
    Clut,
    Matrix,
    Full,
}

pub(crate) fn staged_lab_transform(stages: LabStages) -> Vec<u8> {
    let with_clut = matches!(stages, LabStages::Clut | LabStages::Full);
    let with_matrix = matches!(stages, LabStages::Matrix | LabStages::Full);
    // B applies x² to lightness via parametricCurveType function zero.
    let mut b_lightness = fixed(b"para", &[0x20000]);
    b_lightness.splice(8..8, [0; 4]);
    let mut transform = b"mAB \0\0\0\0\x03\x03\0\0".to_vec();
    transform.resize(32, 0);
    append_stage(&mut transform, 12, [b_lightness, curve(&[]), curve(&[])]);

    if with_matrix {
        // Matrix rows implement [2X - 1/4, Y/4 + 3/8, -Y/4 + Z/8 + 1/2].
        let coefficients: [i32; 12] = [
            131072, 0, 0, 0, 16384, 0, 0, -16384, 8192, -16384, 24576, 32768,
        ];
        let matrix: Vec<_> = coefficients
            .into_iter()
            .flat_map(i32::to_be_bytes)
            .collect();
        append_stage(&mut transform, 16, [matrix]);
        append_stage(
            &mut transform,
            20,
            [curve(&[0x200]), curve(&[]), curve(&[])],
        );
    }
    if with_clut {
        // Distinct grid lengths and a red-green cross term distinguish axis
        // order and multilinear interpolation on the two green cells.
        let mut clut = vec![2, 3, 2];
        clut.resize(16, 0);
        let precision = if with_matrix { 2 } else { 1 };
        clut.extend([precision, 0, 0, 0]);
        for r in [0, 1] {
            for g in [0, 1, 2] {
                for b in [0, 1] {
                    let values = [
                        36 + 46 * r * g + 46 * b + 24 * g,
                        104 + 50 * r + 8 * g,
                        112 + 8 * g + 28 * b,
                    ];
                    if precision == 1 {
                        clut.extend(values);
                    } else {
                        // Include low bits beyond a replicated 8-bit value.
                        clut.extend(
                            values
                                .into_iter()
                                .flat_map(|value| (u16::from(value) * 257 + 199).to_be_bytes()),
                        );
                    }
                }
            }
        }
        append_stage(&mut transform, 24, [clut]);
        append_stage(
            &mut transform,
            28,
            [curve(&[0x200]), curve(&[]), curve(&[])],
        );
    }
    transform
}

fn append_stage<const N: usize>(transform: &mut Vec<u8>, field: usize, elements: [Vec<u8>; N]) {
    let offset = u32::try_from(transform.len()).unwrap();
    transform[field..field + 4].copy_from_slice(&offset.to_be_bytes());
    for element in elements {
        transform.extend(element);
        transform.resize(transform.len().next_multiple_of(4), 0);
    }
}

pub(crate) fn lut_profile_with_transform(transform: Vec<u8>, pcs: &[u8; 4]) -> Vec<u8> {
    let mut profile = v4_profile(b"RGB ", "Surgeist RGB fixture", vec![(*b"A2B0", transform)]);
    profile[20..24].copy_from_slice(pcs);
    profile
}

pub(crate) fn colorimetric_lut_profile(transform: Vec<u8>, pcs: &[u8; 4], version: u8) -> Vec<u8> {
    // Include the mandatory A2B0 transform and exercise A2B1 directly.
    let tags = vec![(*b"A2B0", transform.clone()), (*b"A2B1", transform)];
    let mut profile = match version {
        2 => v2_rgb_profile(tags),
        4 => v4_profile(b"RGB ", "Surgeist colorimetric LUT fixture", tags),
        _ => panic!("unsupported fixture version {version}"),
    };
    profile[20..24].copy_from_slice(pcs);
    profile
}

pub(crate) fn matrix_and_lut_profile(with_a2b1: bool) -> Vec<u8> {
    let mut tags = matrix_tags([RED, GREEN, BLUE], sampled_curve());
    tags.push((*b"A2B0", lut16_transform()));
    if with_a2b1 {
        tags.push((*b"A2B1", lut_a_to_b_transform()));
    }
    v4_profile(b"RGB ", "Surgeist matrix and LUT fixture", tags)
}

fn v2_rgb_profile(mut tags: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    // ICC.1:2001-04 §§6.1–6.3 and 6.5.
    // <https://archive.color.org/files/ICC_Minor_Revision_for_Web.pdf>
    let ascii = b"Surgeist v2 RGB fixture\0";
    let mut description = b"desc\0\0\0\0".to_vec();
    description.extend(u32::try_from(ascii.len()).unwrap().to_be_bytes());
    description.extend_from_slice(ascii);
    // Unused Unicode code/count (8), ScriptCode code/count (3), and text (67).
    description.extend_from_slice(&[0; 8 + 3 + 67]);
    tags.extend([
        (*b"wtpt", fixed(b"XYZ ", &D50)),
        (*b"chad", fixed(b"sf32", &D65_TO_D50)),
        (*b"desc", description),
        (*b"cprt", b"text\0\0\0\0Surgeist test fixture\0".to_vec()),
    ]);
    input_profile([2, 0x40], b"RGB ", tags)
}

fn fixed(kind: &[u8; 4], values: &[i32]) -> Vec<u8> {
    let mut bytes = kind.to_vec();
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend(values.iter().flat_map(|value| value.to_be_bytes()));
    bytes
}

fn mluc(text: &str) -> Vec<u8> {
    let mut bytes = b"mluc\0\0\0\0\0\0\0\x01\0\0\0\x0c\x65\x6e\x55\x53".to_vec();
    let text: Vec<_> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
    bytes.extend(u32::try_from(text.len()).unwrap().to_be_bytes());
    bytes.extend(28_u32.to_be_bytes());
    bytes.extend_from_slice(&text);
    bytes
}

fn v4_profile(
    color_space: &[u8; 4],
    description: &str,
    mut tags: Vec<([u8; 4], Vec<u8>)>,
) -> Vec<u8> {
    tags.extend([
        (*b"wtpt", fixed(b"XYZ ", &D50)),
        (*b"chad", fixed(b"sf32", &D65_TO_D50)),
        (*b"desc", mluc(description)),
        (*b"cprt", mluc("Surgeist test fixture")),
    ]);
    input_profile([4, 0x40], color_space, tags)
}

fn input_profile(
    version: [u8; 2],
    color_space: &[u8; 4],
    tags: Vec<([u8; 4], Vec<u8>)>,
) -> Vec<u8> {
    let mut bytes = vec![0; 132 + 12 * tags.len()];
    bytes[8..10].copy_from_slice(&version);
    bytes[12..16].copy_from_slice(b"scnr");
    bytes[16..20].copy_from_slice(color_space);
    bytes[20..24].copy_from_slice(b"XYZ ");
    bytes[24..36].copy_from_slice(b"\x07\xe9\0\x01\0\x01\0\0\0\0\0\0");
    bytes[36..40].copy_from_slice(b"acsp");
    bytes[47] = 1; // embedded profile flag
    bytes[67] = 1; // media-relative colorimetric
    bytes[68..80].copy_from_slice(&fixed(b"XYZ ", &D50)[8..]);
    bytes[128..132].copy_from_slice(&u32::try_from(tags.len()).unwrap().to_be_bytes());
    for (index, (signature, tag)) in tags.into_iter().enumerate() {
        let entry = 132 + 12 * index;
        let offset = u32::try_from(bytes.len()).unwrap().to_be_bytes();
        bytes[entry..entry + 4].copy_from_slice(&signature);
        bytes[entry + 4..entry + 8].copy_from_slice(&offset);
        bytes[entry + 8..entry + 12]
            .copy_from_slice(&u32::try_from(tag.len()).unwrap().to_be_bytes());
        bytes.extend(tag);
        bytes.resize(bytes.len().next_multiple_of(4), 0);
    }
    let size = u32::try_from(bytes.len()).unwrap().to_be_bytes();
    bytes[..4].copy_from_slice(&size);
    bytes
}

pub(crate) fn image(
    pixels: &[u8],
    profile: Option<&[u8]>,
    before_idat: &[(png::chunk::ChunkType, &[u8])],
    after_idat: &[(png::chunk::ChunkType, &[u8])],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut info = png::Info::with_size(u32::try_from(pixels.len() / 4).unwrap(), 1);
    info.color_type = png::ColorType::Rgba;
    info.icc_profile = profile.map(Cow::Borrowed);
    let mut writer = png::Encoder::with_info(&mut bytes, info)
        .unwrap()
        .write_header()
        .unwrap();
    for (kind, payload) in before_idat {
        writer.write_chunk(*kind, payload).unwrap();
    }
    writer.write_image_data(pixels).unwrap();
    for (kind, payload) in after_idat {
        writer.write_chunk(*kind, payload).unwrap();
    }
    writer.finish().unwrap();
    bytes
}
