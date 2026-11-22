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
        // ICC.1:2022 §10.16. The optional media-relative multi-process
        // route has one 3-to-3 float matrix element with zero offsets.
        let mut element = b"matf\0\0\0\0\0\x03\0\x03".to_vec();
        let xyz = [RED, GREEN, BLUE];
        for row in 0..3 {
            for column in xyz {
                element.extend((column[row] as f32 / 65536.0).to_be_bytes());
            }
        }
        element.extend_from_slice(&[0; 12]);
        let mut transform = b"mpet\0\0\0\0\0\x03\0\x03".to_vec();
        transform.extend_from_slice(&1_u32.to_be_bytes());
        transform.extend_from_slice(&24_u32.to_be_bytes());
        transform.extend_from_slice(&u32::try_from(element.len()).unwrap().to_be_bytes());
        transform.extend(element);
        tags.push((*b"D2B1", transform));
    }
    v4_profile(b"RGB ", "Surgeist piecewise P3 fixture", tags)
}

pub(crate) fn v2_matrix_profile() -> Vec<u8> {
    // ICC.1:2001-04 §§6.1–6.3 and 6.5.
    // <https://archive.color.org/files/ICC_Minor_Revision_for_Web.pdf>
    // The v2 XYZ type requires nonnegative colorants. Use the sRGB primaries
    // from CSS Color 4, here Bradford-adapted to the ICC D50 PCS in s15Fixed16.
    let columns = [
        [28576, 14581, 912],
        [25239, 46983, 6361],
        [9375, 3972, 46787],
    ];
    let mut tags = matrix_tags(columns, sampled_curve());
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
    v4_profile(b"RGB ", "Surgeist RGB fixture", vec![(*b"A2B0", transform)])
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
