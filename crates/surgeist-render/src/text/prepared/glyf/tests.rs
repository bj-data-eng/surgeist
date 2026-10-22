use super::*;
use crate::{GlyphSynthesis, TextInstance};

const FONT: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const X: u16 = 58;
const A: u16 = 35;
fn u16_at(data: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(data[at..at + 2].try_into().unwrap())
}
fn u32_at(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(data[at..at + 4].try_into().unwrap())
}
fn table<'a>(font: &'a [u8], tag: &[u8; 4]) -> &'a [u8] {
    for i in 0..usize::from(u16_at(font, 4)) {
        let at = 12 + i * 16;
        if &font[at..at + 4] == tag {
            let start = usize::try_from(u32_at(font, at + 8)).unwrap();
            let len = usize::try_from(u32_at(font, at + 12)).unwrap();
            return &font[start..start + len];
        }
    }
    panic!("missing table");
}
fn replace(changes: &[(u16, Vec<u8>)]) -> FontData {
    let old_loca = table(FONT, b"loca");
    let old_glyf = table(FONT, b"glyf");
    let count = usize::from(u16_at(table(FONT, b"maxp"), 4));
    let mut loca = Vec::new();
    let mut glyf = Vec::new();
    for id in 0..count {
        loca.extend(u32::try_from(glyf.len()).unwrap().to_be_bytes());
        if let Some((_, bytes)) = changes.iter().find(|(gid, _)| usize::from(*gid) == id) {
            glyf.extend_from_slice(bytes);
        } else {
            let start = usize::from(u16_at(old_loca, id * 2)) * 2;
            let end = usize::from(u16_at(old_loca, id * 2 + 2)) * 2;
            glyf.extend_from_slice(&old_glyf[start..end]);
        }
    }
    loca.extend(u32::try_from(glyf.len()).unwrap().to_be_bytes());
    let count = usize::from(u16_at(FONT, 4));
    let mut result = FONT[..12 + count * 16].to_vec();
    for i in 0..count {
        let at = 12 + i * 16;
        let tag: [u8; 4] = FONT[at..at + 4].try_into().unwrap();
        let mut data = match &tag {
            b"loca" => loca.clone(),
            b"glyf" => glyf.clone(),
            _ => table(FONT, &tag).to_vec(),
        };
        if &tag == b"head" {
            data[50..52].copy_from_slice(&1i16.to_be_bytes());
        }
        while !result.len().is_multiple_of(4) {
            result.push(0);
        }
        let offset = u32::try_from(result.len()).unwrap().to_be_bytes();
        result[at + 8..at + 12].copy_from_slice(&offset);
        result[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        result.extend_from_slice(&data);
    }
    FontData::try_from_bytes(result, 0).unwrap()
}
fn compound(contours: i16, children: &[u16], instruction: Option<&[u8]>) -> Vec<u8> {
    let mut data: Vec<_> = [contours, 0, -200, 1000, 800]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect();
    for (i, &child) in children.iter().enumerate() {
        let last = i + 1 == children.len();
        let flag: u16 = 3 | if last {
            if instruction.is_some() { 256 } else { 0 }
        } else {
            32
        };
        for word in [flag, child, 0, 0] {
            data.extend(word.to_be_bytes());
        }
    }
    if let Some(program) = instruction {
        data.extend(u16::try_from(program.len()).unwrap().to_be_bytes());
        data.extend(program);
    }
    data
}
fn simple(points: u16) -> Vec<u8> {
    assert!(points >= 4);
    let mut data: Vec<_> = [1i16, 0, 0, 100, 100]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect();
    data.extend((points - 1).to_be_bytes());
    data.extend(0u16.to_be_bytes());
    data.extend(std::iter::repeat_n(1u8, usize::from(points)));
    for axis in [[0i16, 100, 0, -100], [0, 0, 100, -100]] {
        for i in 0..usize::from(points) {
            data.extend(axis.get(i).copied().unwrap_or(0).to_be_bytes());
        }
    }
    data
}
fn limits() -> GlyphRunLimits {
    GlyphRunLimits::new(4, 140000, 16 << 20, 32 << 20)
}
fn instance() -> TextInstance {
    TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap()
}
fn glyph(id: u16) -> TextGlyph {
    TextGlyph::try_new(u32::from(id), 0.0, 0.0, 0.0).unwrap()
}
fn prepare(font: &FontData) -> Result<PreparedGlyphRun> {
    font.prepare_glyph_run(&instance(), &[glyph(X)], limits())
}
fn cause(result: Result<PreparedGlyphRun>) -> GlyphPreparationError {
    *result.unwrap_err().glyph_preparation_diagnostic().unwrap()
}
fn unsupported_composite() -> GlyphPreparationError {
    GlyphPreparationError::UnsupportedOutline {
        capability: GlyphCapability::CompositeGlyph,
    }
}

#[test]
fn negative_contour_counts_and_complete_unhinted_program_share_actual_static_geometry() {
    let reference = prepare(&replace(&[(X, compound(-1, &[A], None))])).unwrap();
    for count in [-2, i16::MIN] {
        let actual = prepare(&replace(&[(
            X,
            compound(count, &[A], Some(&[0x40, 0xff, 0x00])),
        )]))
        .unwrap();
        assert_eq!(actual.bounds(), reference.bounds());
        assert_eq!(
            actual.path_elements().collect::<Vec<_>>(),
            reference.path_elements().collect::<Vec<_>>()
        );
    }
}

#[test]
fn actual_simple_terminal_at_provider_depth32_succeeds_but_depth33_refuses() {
    for depth in [32u16, 33] {
        let changes: Vec<_> = (0..depth)
            .map(|i| {
                (
                    X + i,
                    compound(-1, &[if i + 1 == depth { A } else { X + i + 1 }], None),
                )
            })
            .collect();
        let result = prepare(&replace(&changes));
        if depth == 32 {
            assert_eq!(result.unwrap().path_elements().count(), 5);
        } else {
            assert_eq!(cause(result), unsupported_composite());
        }
    }
    // Cache a subtree at shallow depth, then reach that same subtree too deep.
    let mut changes = vec![
        (X, compound(-1, &[59, 60], None)),
        (59, compound(-1, &[A], None)),
    ];
    changes.extend((60u16..91).map(|id| {
        (
            id,
            compound(-1, &[if id == 90 { 59 } else { id + 1 }], None),
        )
    }));
    assert_eq!(cause(prepare(&replace(&changes))), unsupported_composite());
}

#[test]
fn cycles_and_truncated_or_uninterpreted_component_tails_never_publish_empty() {
    for changes in [
        vec![(X, compound(-1, &[X], None))],
        vec![
            (X, compound(-1, &[59], None)),
            (59, compound(-1, &[X], None)),
        ],
    ] {
        assert_eq!(
            cause(prepare(&replace(&changes))),
            GlyphPreparationError::InvalidFont
        );
    }
    let complete = compound(-1, &[A], Some(&[0x40]));
    for end in [10, 11, 13, 15, 17, 18, 19, 20] {
        assert_eq!(
            cause(prepare(&replace(&[(X, complete[..end].to_vec())]))),
            GlyphPreparationError::InvalidFont
        );
    }
    let mut residual = compound(-1, &[A], None);
    residual.extend([0, 0, 1]);
    assert_eq!(
        cause(prepare(&replace(&[(X, residual)]))),
        unsupported_composite()
    );
    let mut empty: Vec<_> = [0i16, 0, 0, 0, 0]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect();
    empty.extend([0, 1, 0x40]);
    let prepared = prepare(&replace(&[
        (X, compound(-1, &[59, 3], None)),
        (59, empty),
        (3, vec![]),
    ]))
    .unwrap();
    assert_eq!(prepared.bounds().kind(), crate::TextRunBoundsKind::Empty);
    assert_eq!(prepared.path_elements().count(), 0);
}

#[test]
fn expanded_real_points_include_every_shared_occurrence_and_four_root_phantoms() {
    for (count, occurrences, accepted) in [
        (65531u16, 1, true),
        (65532, 1, false),
        (32765, 2, true),
        (32766, 2, false),
    ] {
        let font = replace(&[
            (X, compound(-1, &vec![A; occurrences], None)),
            (A, simple(count)),
        ]);
        let result = prepare(&font);
        if accepted {
            let prepared = result.unwrap();
            assert_eq!(prepared.bounds().ink_rect().unwrap().width(), 100.0);
            assert_eq!(
                prepared
                    .path_elements()
                    .filter(|p| matches!(p, PathElement::MoveTo(_)))
                    .count(),
                occurrences
            );
        } else {
            assert_eq!(cause(result), unsupported_composite());
        }
    }
}

#[test]
fn closure_storage_and_path_refusal_preserve_actual_inputs_for_same_owner_retry() {
    let font = replace(&[
        (X, compound(-1, &[59, 59], None)),
        (59, compound(-1, &[A], None)),
    ]);
    let original = font.clone();
    let instance = instance();
    let glyphs = [glyph(A), glyph(X)];
    let ready = font
        .prepare_glyph_run(&instance, &glyphs, limits())
        .unwrap();
    for tight in [
        GlyphRunLimits::new(4, 140000, 128, 32 << 20),
        GlyphRunLimits::new(4, 9, 16 << 20, 32 << 20),
        GlyphRunLimits::new(4, 140000, 16 << 20, 0),
    ] {
        let error = font
            .prepare_glyph_run(&instance, &glyphs, tight)
            .unwrap_err();
        assert!(matches!(
            error.glyph_preparation_diagnostic(),
            Some(GlyphPreparationError::Limit { .. })
        ));
        assert_eq!(font, original);
        let retried = font
            .prepare_glyph_run(&instance, &glyphs, limits())
            .unwrap();
        assert_eq!(retried.bounds(), ready.bounds());
        assert_eq!(
            retried.path_elements().collect::<Vec<_>>(),
            ready.path_elements().collect::<Vec<_>>()
        );
        assert!(!retried.is_same_preparation(&ready));
    }
}

#[test]
fn explicit_identity_encodings_and_first_overlap_keep_ink_while_ambiguous_records_refuse() {
    let reference = prepare(&replace(&[(X, compound(-1, &[A], None))])).unwrap();
    for (flags, values) in [
        (3 | 8, vec![16384i16]),
        (3 | 64, vec![16384, 16384]),
        (3 | 128, vec![16384, 0, 0, 16384]),
        (3 | 4 | 2048, vec![]),
        (3 | 4096 | 1024, vec![]),
    ] {
        let mut data = compound(-1, &[A], None);
        data[10..12].copy_from_slice(&(flags as u16).to_be_bytes());
        data.extend(values.into_iter().flat_map(i16::to_be_bytes));
        let actual = prepare(&replace(&[(X, data)])).unwrap();
        assert_eq!(actual.bounds(), reference.bounds());
        assert_eq!(
            actual.path_elements().collect::<Vec<_>>(),
            reference.path_elements().collect::<Vec<_>>()
        );
    }
    for (flags, expected) in [
        (3 | 8 | 64, GlyphPreparationError::InvalidFont),
        (3 | 2048 | 4096, GlyphPreparationError::InvalidFont),
        (3 | 0x8000, unsupported_composite()),
        (3 | 0x10, unsupported_composite()),
    ] {
        let mut data = compound(-1, &[A], None);
        data[10..12].copy_from_slice(&(flags as u16).to_be_bytes());
        assert_eq!(cause(prepare(&replace(&[(X, data)]))), expected);
    }
    let mut data = compound(-1, &[A, A], None);
    data[18..20].copy_from_slice(&(3u16 | 1024).to_be_bytes());
    assert_eq!(
        cause(prepare(&replace(&[(X, data.clone())]))),
        unsupported_composite()
    );
    data[10..12].copy_from_slice(&(35u16 | 1024).to_be_bytes());
    assert_eq!(
        prepare(&replace(&[(X, data)]))
            .unwrap()
            .path_elements()
            .count(),
        10
    );
}

#[test]
fn malformed_child_reference_and_root_metric_extent_refuse_before_provider_fallback() {
    assert_eq!(
        cause(prepare(&replace(&[(X, compound(-1, &[u16::MAX], None))]))),
        GlyphPreparationError::InvalidFont
    );
    let good = replace(&[(X, compound(-1, &[A], None))]);
    let mut bytes = good.bytes().to_vec();
    for i in 0..usize::from(u16_at(&bytes, 4)) {
        let at = 12 + i * 16;
        if &bytes[at..at + 4] == b"hmtx" {
            bytes[at + 12..at + 16].copy_from_slice(&2u32.to_be_bytes());
        }
    }
    let font = FontData::try_from_bytes(bytes, 0).unwrap();
    assert_eq!(cause(prepare(&font)), GlyphPreparationError::InvalidFont);
}

#[test]
fn simultaneous_old_new_scratch_capacity_is_admitted_before_growth() {
    let mut values = Vec::<u64>::with_capacity(3);
    values.extend([7, 11, 13]);
    let old_capacity = values.capacity();
    let other = 128;
    let coordinates = 64;
    // Final backing alone fits. Its still-live old allocation must also fit
    // before the reservation that may replace it.
    let final_only = other + coordinates + (old_capacity + 1) * size_of::<u64>();
    let error = grow(&mut values, other, coordinates, final_only).unwrap_err();
    assert!(matches!(
        error.glyph_preparation_diagnostic(),
        Some(GlyphPreparationError::Limit {
            resource: GlyphResource::ScratchBytes,
            ..
        })
    ));
    assert_eq!(values.capacity(), old_capacity);
    assert_eq!(values, [7, 11, 13]);
    grow(
        &mut values,
        other,
        coordinates,
        final_only + old_capacity * size_of::<u64>(),
    )
    .unwrap();
    values.push(17);
    assert_eq!(values, [7, 11, 13, 17]);
}

#[test]
fn actual_fvar_presence_keeps_static_composite_outside_the_admitted_domain() {
    let original = replace(&[(X, compound(-1, &[A], None))]);
    let source = original.bytes();
    let mut records: Vec<([u8; 4], Vec<u8>)> = (0..usize::from(u16_at(source, 4)))
        .map(|i| {
            let tag = source[12 + i * 16..16 + i * 16].try_into().unwrap();
            (tag, table(source, &tag).to_vec())
        })
        .collect();
    // Complete fvar with zero axes/instances: even a source-qualified default
    // is not permission to run a composite through this static-only entry.
    let mut fvar = 0x00010000u32.to_be_bytes().to_vec();
    for value in [16u16, 2, 0, 20, 0, 4] {
        fvar.extend(value.to_be_bytes());
    }
    records.push((*b"fvar", fvar));
    records.sort_by_key(|(tag, _)| *tag);
    let n = records.len();
    let mut bytes = vec![0; 12 + n * 16];
    bytes[..4].copy_from_slice(&source[..4]);
    bytes[4..6].copy_from_slice(&u16::try_from(n).unwrap().to_be_bytes());
    let exponent = n.ilog2();
    let search = (1usize << exponent) * 16;
    bytes[6..8].copy_from_slice(&u16::try_from(search).unwrap().to_be_bytes());
    bytes[8..10].copy_from_slice(&u16::try_from(exponent).unwrap().to_be_bytes());
    bytes[10..12].copy_from_slice(&u16::try_from(n * 16 - search).unwrap().to_be_bytes());
    for (i, (tag, data)) in records.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let offset = u32::try_from(bytes.len()).unwrap();
        let at = 12 + i * 16;
        bytes[at..at + 4].copy_from_slice(&tag);
        bytes[at + 8..at + 12].copy_from_slice(&offset.to_be_bytes());
        bytes[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend(data);
    }
    let font = FontData::try_from_bytes(bytes, 0).unwrap();
    assert_eq!(cause(prepare(&font)), unsupported_composite());
    assert!(prepare(&original).is_ok());
}

#[test]
fn actual_draw_depth_counts_empty_loca_loads_even_after_shallow_memoization() {
    let mut failures = Vec::new();
    for depth in [32u16, 33] {
        for (label, root_children) in [
            ("pointful root", vec![A, X + 1]),
            ("cached empty child", vec![3, A, X + 1]),
            ("source-proved empty root", vec![X + 1]),
        ] {
            let mut changes = vec![(X, compound(-1, &root_children, None))];
            for i in 1..depth {
                changes.push((
                    X + i,
                    compound(-1, &[if i + 1 == depth { 3 } else { X + i + 1 }], None),
                ));
            }
            let result = prepare(&replace(&changes));
            if depth == 32 {
                let output = result.unwrap();
                if label == "source-proved empty root" {
                    assert_eq!(output.bounds().kind(), crate::TextRunBoundsKind::Empty);
                } else {
                    assert_eq!(output.path_elements().count(), 5);
                }
            } else {
                match result {
                    Err(error)
                        if error.glyph_preparation_diagnostic()
                            == Some(&unsupported_composite()) => {}
                    Err(error) => failures.push(format!(
                        "{label} depth{depth}: {:?}",
                        error.glyph_preparation_diagnostic()
                    )),
                    Ok(output) => failures.push(format!(
                        "{label} depth{depth}: unexpected {:?}",
                        output.bounds()
                    )),
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
