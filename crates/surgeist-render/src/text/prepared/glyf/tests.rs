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
    FontData::try_from_bytes(result, 0, surgeist_test::fixtures::FONT_LIMITS).unwrap()
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
    font.prepare_glyph_run(
        &instance(),
        &[glyph(X)],
        limits(),
        &mut crate::RenderAccountingContext::default(),
    )
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
        .prepare_glyph_run(
            &instance,
            &glyphs,
            limits(),
            &mut crate::RenderAccountingContext::default(),
        )
        .unwrap();
    for tight in [
        GlyphRunLimits::new(4, 140000, 128, 32 << 20),
        GlyphRunLimits::new(4, 9, 16 << 20, 32 << 20),
        GlyphRunLimits::new(4, 140000, 16 << 20, 0),
    ] {
        let error = font
            .prepare_glyph_run(
                &instance,
                &glyphs,
                tight,
                &mut crate::RenderAccountingContext::default(),
            )
            .unwrap_err();
        assert!(matches!(
            error.glyph_preparation_diagnostic(),
            Some(GlyphPreparationError::Limit { .. })
        ));
        assert_eq!(font, original);
        let retried = font
            .prepare_glyph_run(
                &instance,
                &glyphs,
                limits(),
                &mut crate::RenderAccountingContext::default(),
            )
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
    let font = FontData::try_from_bytes(bytes, 0, surgeist_test::fixtures::FONT_LIMITS).unwrap();
    assert_eq!(cause(prepare(&font)), GlyphPreparationError::InvalidFont);
}

#[test]
fn simultaneous_old_new_scratch_capacity_is_admitted_before_growth() {
    let mut context = RenderAccountingContext::default();
    let old = 3 * size_of::<u64>();
    let final_only = 128 + 64 + 4 * size_of::<u64>();
    context
        .prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, final_only, 0),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let _other = account.acquire(RenderResource::ScratchCpuBytes, 128 + 64)?;
                let mut values = account.temporary::<u64>(3)?;
                values.extend_from_slice(&[7, 11, 13])?;
                let error = values.grow(1).unwrap_err();
                assert!(matches!(
                    error.glyph_preparation_diagnostic(),
                    Some(GlyphPreparationError::Limit {
                        resource: GlyphResource::ScratchBytes,
                        ..
                    })
                ));
                assert_eq!(values.as_slice(), [7, 11, 13]);
                assert_eq!(values.capacity(), 3);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .peak(RenderResource::ScratchCpuBytes),
        (128 + 64 + old) as u64
    );
    context
        .prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, final_only + old, 0),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let _other = account.acquire(RenderResource::ScratchCpuBytes, 128 + 64)?;
                let mut values = account.temporary::<u64>(3)?;
                values.extend_from_slice(&[7, 11, 13])?;
                values.grow(1)?;
                values.push(17)?;
                assert_eq!(values.as_slice(), [7, 11, 13, 17]);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .peak(RenderResource::ScratchCpuBytes),
        (final_only + old) as u64
    );
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
    let font = FontData::try_from_bytes(bytes, 0, surgeist_test::fixtures::FONT_LIMITS).unwrap();
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

// Four explicit on-curve points: (0,0),(100,0),(100,100),(0,100).
fn square() -> Vec<u8> {
    let mut data = simple(4);
    let end = data.len();
    data[end - 2..].copy_from_slice(&0i16.to_be_bytes());
    data
}

fn literal_square(x: f64, y: f64) -> [PathElement; 5] {
    [
        PathElement::MoveTo(Point::new(x, y)),
        PathElement::LineTo(Point::new(x + 100.0, y)),
        PathElement::LineTo(Point::new(x + 100.0, y - 100.0)),
        PathElement::LineTo(Point::new(x, y - 100.0)),
        PathElement::Close,
    ]
}

struct ClosureCase {
    changes: Vec<(u16, Vec<u8>)>,
    copies: usize,
    visits: u64,
    edges: u64,
}
fn closure_cases() -> [ClosureCase; 4] {
    [
        (vec![(X, compound(-1, &[A], None))], 1, 4, 1),
        (vec![(X, compound(-1, &[A, A], None))], 2, 5, 2),
        (
            vec![
                (X, compound(-1, &[59], None)),
                (59, compound(-1, &[A], None)),
            ],
            1,
            7,
            2,
        ),
        (
            vec![
                (X, compound(-1, &[59, 59], None)),
                (59, compound(-1, &[A], None)),
            ],
            2,
            9,
            3,
        ),
    ]
    .map(|(changes, copies, visits, edges)| ClosureCase {
        changes,
        copies,
        visits,
        edges,
    })
}

#[test]
fn static_closure_paths_follow_literal_source_points_for_every_occurrence() {
    for ClosureCase {
        mut changes,
        copies,
        ..
    } in closure_cases()
    {
        changes.push((A, square()));
        let font = replace(&changes);
        let output = prepare(&font).unwrap();
        let expected: Vec<_> = literal_square(0.0, 0.0).repeat(copies);
        assert_eq!(output.path_elements().collect::<Vec<_>>(), expected);
        assert_eq!(
            output.bounds().ink_rect(),
            Some(Rect::new(0.0, -100.0, 100.0, 100.0))
        );
    }
    let mut nested = compound(-1, &[A], None);
    nested[14..16].copy_from_slice(&10i16.to_be_bytes());
    nested[16..18].copy_from_slice(&20i16.to_be_bytes());
    let mut root = compound(-1, &[59, 59], None);
    root[22..24].copy_from_slice(&200i16.to_be_bytes());
    let output = prepare(&replace(&[(A, square()), (59, nested), (X, root)])).unwrap();
    let expected: Vec<_> = literal_square(10.0, -20.0)
        .into_iter()
        .chain(literal_square(210.0, -20.0))
        .collect();
    assert_eq!(output.path_elements().collect::<Vec<_>>(), expected);
    assert_eq!(
        output.bounds().ink_rect(),
        Some(Rect::new(10.0, -120.0, 300.0, 100.0))
    );
}

fn isolated(font: &FontData, context: &mut RenderAccountingContext) -> Result<Summary> {
    let font = skrifa::FontRef::new(font.bytes()).unwrap();
    context.prepare(
        RenderAttemptKind::GlyphRun,
        limits(),
        #[cfg(feature = "render-text")]
        None,
        |account| {
            account.phase(RenderPhase::Qualify, || {
                qualify(&font, GlyphId::new(u32::from(X)), &[], account)
            })
        },
    )
}

#[test]
fn isolated_closure_probes_count_only_real_cache_active_and_raw_work() {
    for ClosureCase {
        changes,
        copies,
        visits,
        edges,
    } in closure_cases()
    {
        let font = replace(&changes);
        let mut context = RenderAccountingContext::default();
        let summary = isolated(&font, &mut context).unwrap();
        assert_eq!(summary.points, copies * 4);
        let report = context.last_attempt().unwrap();
        assert_eq!(report.cleanup_fault(), None);
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::GlyphVisits), visits);
        assert_eq!(usage.work(RenderWork::GlyphComponentVisits), edges);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    }
}

fn capacity<T>(requested: usize) -> usize {
    let mut backing = Vec::<T>::new();
    backing.try_reserve_exact(requested).unwrap();
    backing.capacity()
}

fn simple_scratch() -> usize {
    // Four actual points plus four provider phantoms, each actual backing.
    point_backings(8).iter().sum()
}

#[test]
fn isolated_nested_closure_admits_actual_backing_overlap_and_relocations() {
    let font = replace(&[
        (X, compound(-1, &[59], None)),
        (59, compound(-1, &[A], None)),
    ]);
    let mut context = RenderAccountingContext::default();
    isolated(&font, &mut context).unwrap();
    // C's real entry reserves one Q; A needs two Q slots while C remains
    // outstanding. Both cache and parent backing use returned capacities.
    let q1 = capacity::<QualifiedGlyph>(1);
    let q2 = if q1 >= 2 {
        q1
    } else {
        capacity::<QualifiedGlyph>(2)
    };
    let p1 = capacity::<AwaitingParent<'_>>(1);
    let q = size_of::<QualifiedGlyph>();
    let p = size_of::<AwaitingParent<'_>>();
    let growth = if q1 >= 2 { q1 } else { q1 + q2 };
    let expected_peak = (q1 * q + p1 * p + simple_scratch()).max(growth * q + p1 * p);
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(
        usage.peak(RenderResource::ScratchCpuBytes),
        expected_peak as u64
    );
    assert_eq!(usage.work(RenderWork::StorageRelocations), 0);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    eprintln!(
        "qualified layout Q={q}, awaiting P={p}, peak={expected_peak}, capacities={q1},{q2}/{p1}, relocations=0"
    );
}

#[test]
fn combined_source_faults_keep_edge_raw_depth_and_inclusion_order() {
    let mut edge_cycle = compound(-1, &[X], None);
    edge_cycle[10..12].copy_from_slice(&(3u16 | 0x8000).to_be_bytes());
    assert_eq!(
        cause(prepare(&replace(&[(X, edge_cycle)]))),
        unsupported_composite()
    );
    // Both raw references and cycles must win over the next depth refusal.
    for child in [u16::MAX, X] {
        let changes: Vec<_> = (X..X + 33)
            .map(|id| {
                (
                    id,
                    compound(-1, &[if id == X + 32 { child } else { id + 1 }], None),
                )
            })
            .collect();
        assert_eq!(
            cause(prepare(&replace(&changes))),
            GlyphPreparationError::InvalidFont
        );
    }
    // The last child's expanded-point refusal precedes a malformed program tail.
    let mut root = compound(-1, &[A], Some(&[0x40]));
    root.pop();
    assert_eq!(
        cause(prepare(&replace(&[(X, root), (A, simple(65532))]))),
        unsupported_composite()
    );
}

fn assert_closed(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    for resource in [
        RenderResource::ScratchCpuBytes,
        RenderResource::RetainedCpuBytes,
    ] {
        assert_eq!(usage.live(resource), 0);
    }
    assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(usage.work(RenderWork::PathElementsEmitted), 0);
}

fn forbidden() -> std::time::Duration {
    panic!("untimed qualification read the clock")
}
fn host_clock() -> std::time::Duration {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed()
}

fn profiles() -> [crate::RenderProfile; 6] {
    use crate::{RenderPhaseFilter, RenderProfile};
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 3,
            filter: RenderPhaseFilter::Phase(RenderPhase::Qualify),
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::Phase(RenderPhase::Preparation),
        },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 3,
            filter: RenderPhaseFilter::All,
        },
    ]
}

#[test]
fn every_probe_and_edge_cap_preserves_literal_admitted_prefix_across_profiles() {
    use crate::{
        RenderAccountingError, RenderAccountingOptions, RenderClock, RenderLimits, RenderQuantity,
        RenderRefusalReason,
    };
    // V=one real glyph probe, E=one edge admission. These are the independently
    // enumerated source/cache/active traces, with no completion probes.
    let traces: [&[u8]; 4] = [b"VEVVV", b"VEVVVEV", b"VEVVEVVVV", b"VEVVEVVVVEVV"];
    for (ClosureCase { changes, .. }, trace) in closure_cases().into_iter().zip(traces) {
        let font = replace(&changes);
        let original = font.clone();
        let mut observed = std::collections::BTreeMap::new();
        for (profile_index, profile) in profiles().into_iter().enumerate() {
            for (unit, work, other) in [
                (
                    b'V',
                    RenderWork::GlyphVisits,
                    RenderWork::GlyphComponentVisits,
                ),
                (
                    b'E',
                    RenderWork::GlyphComponentVisits,
                    RenderWork::GlyphVisits,
                ),
            ] {
                let total = trace.iter().filter(|&&v| v == unit).count();
                for cap in 0..=total {
                    let timed = profile_index >= 4;
                    let options = RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_clock(RenderClock::Host(if timed {
                            host_clock
                        } else {
                            forbidden
                        }))
                        .with_limits(
                            RenderLimits::new()
                                .with_work_limit(work, cap as u64)
                                .unwrap(),
                        );
                    let mut context = RenderAccountingContext::new(options);
                    let result = isolated(&font, &mut context);
                    let report = context.last_attempt().unwrap().attempt().unwrap();
                    let usage = report.usage();
                    if cap == total {
                        result.unwrap();
                        assert_eq!(report.refusal(), None);
                        assert_eq!(
                            usage.work(other),
                            trace.iter().filter(|&&v| v != unit).count() as u64
                        );
                    } else {
                        let error = result.unwrap_err();
                        let Some(RenderAccountingError::Refused(refusal)) =
                            error.accounting_diagnostic()
                        else {
                            panic!("work cap must retain its accounting cause: {error:?}");
                        };
                        assert_eq!(refusal.phase(), RenderPhase::Attempt);
                        assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
                        assert_eq!(refusal.current(), cap as u64);
                        assert_eq!(refusal.amount(), 1);
                        assert_eq!(
                            refusal.reason(),
                            RenderRefusalReason::Limit { limit: cap as u64 }
                        );
                        assert_eq!(report.refusal(), Some(refusal));
                        let denied_at = trace
                            .iter()
                            .enumerate()
                            .filter(|(_, v)| **v == unit)
                            .nth(cap)
                            .unwrap()
                            .0;
                        assert_eq!(
                            usage.work(other),
                            trace[..denied_at].iter().filter(|&&v| v != unit).count() as u64
                        );
                    }
                    // Detail selection and clock reads cannot alter any owned
                    // work, peak backing, refusal prefix or closing custody.
                    if profile_index == 0 {
                        observed.insert((unit, cap), usage);
                    } else {
                        assert_eq!(usage, observed[&(unit, cap)]);
                    }
                    assert_eq!(usage.work(work), cap as u64);
                    assert_eq!(report.elapsed().is_some(), timed);
                    assert_closed(&context);
                    assert_eq!(font, original);
                }
            }
        }
        // Same original owner/instance/glyphs remain usable after all refusals.
        assert_eq!(
            prepare(&font).unwrap().bounds().ink_rect(),
            Some(Rect::new(0.0, -800.0, 1000.0, 1000.0))
        );
    }
}

#[test]
fn completed_miss_is_charged_before_root_to_leaf_cycle_detection() {
    use crate::{RenderAccountingOptions, RenderLimits};
    let font = replace(&[(X, compound(-1, &[A, X], None))]);
    // Root1 + first child(active1,raw2) + completed A miss1 + active root1.
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::GlyphVisits, 5)
                .unwrap(),
        ),
    );
    let error = isolated(&font, &mut context).unwrap_err();
    assert!(error.accounting_diagnostic().is_some());
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::GlyphVisits),
        5
    );
    assert_closed(&context);
    context.set_options(RenderAccountingOptions::new());
    assert_eq!(
        isolated(&font, &mut context)
            .unwrap_err()
            .glyph_preparation_diagnostic(),
        Some(&GlyphPreparationError::InvalidFont)
    );
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::GlyphVisits),
        6
    );
    assert_closed(&context);
}

fn point_backings(points: usize) -> [usize; 3] {
    [
        capacity::<skrifa::raw::types::Point<i32>>(points)
            * size_of::<skrifa::raw::types::Point<i32>>(),
        capacity::<skrifa::raw::tables::glyf::PointFlags>(points)
            * size_of::<skrifa::raw::tables::glyf::PointFlags>(),
        capacity::<skrifa::raw::types::Point<f32>>(points)
            * size_of::<skrifa::raw::types::Point<f32>>(),
    ]
}

#[test]
fn root_current_is_inline_and_empty_children_still_qualify_without_fictional_root_storage() {
    use crate::{RenderAccountingOptions, RenderLimits};
    for (changes, points, cache_bytes) in [
        (vec![(X, square())], 8, 0),
        (
            vec![(X, compound(-1, &[3], None)), (3, vec![])],
            4,
            capacity::<QualifiedGlyph>(1) * size_of::<QualifiedGlyph>(),
        ),
    ] {
        let font = replace(&changes);
        let expected = point_backings(points)
            .iter()
            .sum::<usize>()
            .max(cache_bytes);
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(expected as u64)),
        );
        let summary = isolated(&font, &mut context).unwrap();
        assert_eq!(summary.points, if points == 8 { 4 } else { 0 });
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), expected as u64);
        assert_eq!(usage.work(RenderWork::StorageRelocations), 0);
        assert_closed(&context);
    }
    let mut invalid_header = compound(-1, &[A], None);
    invalid_header[2..4].copy_from_slice(&2000i16.to_be_bytes());
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_limits(RenderLimits::new().with_scratch_bytes(0)),
    );
    assert_eq!(
        isolated(&replace(&[(X, invalid_header)]), &mut context)
            .unwrap_err()
            .glyph_preparation_diagnostic(),
        Some(&GlyphPreparationError::InvalidFont)
    );
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .peak(RenderResource::ScratchCpuBytes),
        0
    );
    assert_closed(&context);
}

#[test]
fn outstanding_composite_slots_cover_simple_appends_completion_overlap_and_retry() {
    use crate::{
        RenderAccountingError, RenderAccountingOptions, RenderChangeKind, RenderEvent,
        RenderLimits, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderRefusalReason,
    };
    let font = replace(&[
        (X, compound(-1, &[59], None)),
        (59, compound(-1, &[A, A + 1, A + 2], None)),
        (A, square()),
        (A + 1, square()),
        (A + 2, square()),
    ]);
    let original = font.clone();
    let q = size_of::<QualifiedGlyph>();
    let p = capacity::<AwaitingParent<'_>>(1) * size_of::<AwaitingParent<'_>>();
    let first_capacity = capacity::<QualifiedGlyph>(1);
    let mut cache_capacity = first_capacity;
    // Explicit operation oracle: C entry; parent push; each leaf's three
    // provider buffers; then its reservation. Only completed leaves relocate.
    let mut acquisitions = vec![(0usize, first_capacity * q), (first_capacity * q, p)];
    let mut relocations = 0u64;
    for (completed_leaves, requested) in [(0, 2), (1, 3), (2, 4)] {
        let mut live = cache_capacity * q + p;
        for amount in point_backings(8) {
            acquisitions.push((live, amount));
            live += amount;
        }
        if cache_capacity < requested {
            let next = capacity::<QualifiedGlyph>(requested);
            acquisitions.push((cache_capacity * q + p, next * q));
            relocations += completed_leaves;
            cache_capacity = next;
        }
    }
    let peak = acquisitions
        .iter()
        .map(|(live, amount)| live + amount)
        .max()
        .unwrap();
    let options = RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
        timing: false,
        event_limit: 512,
        filter: RenderPhaseFilter::All,
    });
    let mut context = RenderAccountingContext::new(
        options.with_limits(RenderLimits::new().with_scratch_bytes(peak as u64)),
    );
    assert_eq!(isolated(&font, &mut context).unwrap().points, 12);
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(
        report.usage().peak(RenderResource::ScratchCpuBytes),
        peak as u64
    );
    assert_eq!(
        report.usage().work(RenderWork::StorageRelocations),
        relocations
    );
    let actual: Vec<_> = report
        .events()
        .filter_map(|event| match event {
            RenderEvent::Change {
                quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                kind: RenderChangeKind::Acquire,
                amount,
                ..
            } if amount > 0 => Some(amount),
            _ => None,
        })
        .collect();
    assert_eq!(
        actual,
        acquisitions
            .iter()
            .map(|(_, amount)| *amount as u64)
            .collect::<Vec<_>>()
    );
    assert_eq!(report.dropped_events(), 0);
    assert_closed(&context);
    // Every distinct acquisition frontier, including old+new cache overlap,
    // is denied one byte short. The denied amount never enters custody.
    for cap in acquisitions.iter().map(|(live, amount)| live + amount - 1) {
        let &(current, amount) = acquisitions
            .iter()
            .find(|(live, amount)| live + amount > cap)
            .unwrap();
        context
            .set_options(options.with_limits(RenderLimits::new().with_scratch_bytes(cap as u64)));
        let error = isolated(&font, &mut context).unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("{error:?}")
        };
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes)
        );
        assert_eq!(refusal.current(), current as u64);
        assert_eq!(refusal.amount(), amount as u64);
        assert_eq!(
            refusal.reason(),
            RenderRefusalReason::Limit { limit: cap as u64 }
        );
        assert_closed(&context);
        assert_eq!(font, original);
    }
    // Relocation refusal preserves old+new ownership until both are destroyed.
    if relocations > 0 {
        context.set_options(
            options.with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::StorageRelocations, 0)
                    .unwrap(),
            ),
        );
        let error = isolated(&font, &mut context).unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("{error:?}")
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::StorageRelocations)
        );
        assert_eq!(refusal.current(), 0);
        assert_eq!(refusal.amount(), 1);
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_closed(&context);
    }
    context.set_options(options);
    assert_eq!(isolated(&font, &mut context).unwrap().points, 12);
    assert_closed(&context);
    let output = prepare(&font).unwrap();
    assert_eq!(
        output.path_elements().collect::<Vec<_>>(),
        literal_square(0.0, 0.0).repeat(3)
    );
    assert_eq!(
        output.bounds().ink_rect(),
        Some(Rect::new(0.0, -100.0, 100.0, 100.0))
    );
}

#[test]
fn parent_replacement_moves_only_suspended_ancestors_and_refusal_releases_both_backings() {
    use crate::{RenderAccountingError, RenderAccountingOptions, RenderLimits, RenderQuantity};
    let font = replace(&[
        (X, compound(-1, &[59], None)),
        (59, compound(-1, &[60], None)),
        (60, compound(-1, &[61], None)),
        (61, compound(-1, &[A], None)),
    ]);
    let q = size_of::<QualifiedGlyph>();
    let p = size_of::<AwaitingParent<'_>>();
    let mut qc = 0;
    let mut pc = 0;
    let mut peak = 0;
    let mut moves = 0;
    let mut before_first_move = None;
    // Explicit three nonroot composite entries. No qualified records exist
    // yet; only parent lengths 0,1,2 can move during these replacements.
    for outstanding in 1..=3 {
        if qc < outstanding {
            let next = capacity::<QualifiedGlyph>(outstanding);
            peak = peak.max((qc + next) * q + pc * p);
            qc = next;
        }
        if pc < outstanding {
            let next = capacity::<AwaitingParent<'_>>(outstanding);
            peak = peak.max(qc * q + (pc + next) * p);
            if outstanding > 1 {
                before_first_move.get_or_insert(peak);
                moves += outstanding - 1;
            }
            pc = next;
        }
    }
    peak = peak.max(qc * q + pc * p + simple_scratch());
    if qc < 4 {
        peak = peak.max((qc + capacity::<QualifiedGlyph>(4)) * q + pc * p);
    }
    let mut context = RenderAccountingContext::default();
    isolated(&font, &mut context).unwrap();
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), peak as u64);
    assert_eq!(usage.work(RenderWork::StorageRelocations), moves as u64);
    assert_closed(&context);
    if let Some(expected_peak) = before_first_move {
        context.set_options(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::StorageRelocations, 0)
                    .unwrap(),
            ),
        );
        let error = isolated(&font, &mut context).unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("{error:?}")
        };
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::StorageRelocations)
        );
        assert_eq!(refusal.current(), 0);
        assert_eq!(refusal.amount(), 1);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(
            usage.peak(RenderResource::ScratchCpuBytes),
            expected_peak as u64
        );
        assert_eq!(usage.work(RenderWork::StorageRelocations), 0);
        assert_closed(&context);
    }
}

#[test]
fn nonroot_composite_reserves_its_real_future_slot_before_header_validation() {
    use crate::{RenderAccountingOptions, RenderLimits};
    let mut malformed = compound(-1, &[A], None);
    malformed[2..4].copy_from_slice(&2000i16.to_be_bytes());
    let font = replace(&[(X, compound(-1, &[59], None)), (59, malformed)]);
    let needed = capacity::<QualifiedGlyph>(1) * size_of::<QualifiedGlyph>();
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_scratch_bytes((needed - 1) as u64)),
    );
    assert!(
        isolated(&font, &mut context)
            .unwrap_err()
            .accounting_diagnostic()
            .is_some()
    );
    assert_closed(&context);
    context.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_scratch_bytes(needed as u64)),
    );
    assert_eq!(
        isolated(&font, &mut context)
            .unwrap_err()
            .glyph_preparation_diagnostic(),
        Some(&GlyphPreparationError::InvalidFont)
    );
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), needed as u64);
    assert_closed(&context);
}

#[test]
fn nested_scratch_overlap_success_refusal_and_same_context_retry_are_profile_independent() {
    use crate::{
        RenderAccountingError, RenderAccountingOptions, RenderClock, RenderLimits, RenderQuantity,
        RenderRefusalReason,
    };
    let font = replace(&[
        (X, compound(-1, &[59], None)),
        (59, compound(-1, &[A], None)),
        (A, square()),
    ]);
    let original = font.clone();
    let q1 = capacity::<QualifiedGlyph>(1);
    let q2 = capacity::<QualifiedGlyph>(2);
    let q = size_of::<QualifiedGlyph>();
    let p = capacity::<AwaitingParent<'_>>(1) * size_of::<AwaitingParent<'_>>();
    let live = q1 * q + p;
    let overlap = if q1 < 2 { (q1 + q2) * q + p } else { live };
    let buffers = point_backings(8);
    let expected_peak = overlap.max(live + buffers.iter().sum::<usize>());
    // Resource refusal at the largest actual nested-closure frontier.
    let cap = expected_peak - 1;
    let mut frontiers = vec![(0, q1 * q), (q1 * q, p)];
    let mut current = live;
    for amount in buffers {
        frontiers.push((current, amount));
        current += amount;
    }
    if q1 < 2 {
        frontiers.push((live, q2 * q));
    }
    let &(current, amount) = frontiers
        .iter()
        .find(|(live, amount)| live + amount > cap)
        .unwrap();
    let mut baseline = None;
    for (index, profile) in profiles().into_iter().enumerate() {
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(if index >= 4 {
                host_clock
            } else {
                forbidden
            }));
        let mut context = RenderAccountingContext::new(
            options.with_limits(RenderLimits::new().with_scratch_bytes(cap as u64)),
        );
        let error = isolated(&font, &mut context).unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("{error:?}")
        };
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes)
        );
        assert_eq!(refusal.current(), current as u64);
        assert_eq!(refusal.amount(), amount as u64);
        assert_eq!(
            refusal.reason(),
            RenderRefusalReason::Limit { limit: cap as u64 }
        );
        let refused = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_closed(&context);
        assert_eq!(font, original);
        context.set_options(
            options.with_limits(RenderLimits::new().with_scratch_bytes(expected_peak as u64)),
        );
        assert_eq!(isolated(&font, &mut context).unwrap().points, 4);
        let succeeded = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(
            succeeded.peak(RenderResource::ScratchCpuBytes),
            expected_peak as u64
        );
        assert_closed(&context);
        if let Some(expected) = baseline {
            assert_eq!((refused, succeeded), expected);
        } else {
            baseline = Some((refused, succeeded));
        }
        let instance = instance();
        let glyphs = [glyph(X)];
        let output = font
            .prepare_glyph_run(&instance, &glyphs, limits(), &mut context)
            .unwrap();
        assert_eq!(
            output.path_elements().collect::<Vec<_>>(),
            literal_square(0.0, 0.0)
        );
    }
}
