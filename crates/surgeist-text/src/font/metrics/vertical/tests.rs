//! Independent registered-font observations; expected values come from authored
//! SFNT design units and the adopted ascent-reference policy, not this resolver.

use crate::test_fonts::{remove_tables, replace_tables, table, u16_at};
use crate::vertical_test_fonts as fixture;
use crate::*;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

use FontLineMetricSource as L;
use VerticalGlyphMetricProvenance as P;
const X: u32 = 58;

fn registered(bytes: Vec<u8>) -> (System, FontData) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let data = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(data.index(), 0);
    assert_eq!(data.bytes(), bytes);
    (system, data)
}
fn query(data: &FontData, glyph: u32, size: f32) -> Result<VerticalGlyphMetrics<'_>> {
    data.unsynthesized_instance(size, &[])?
        .vertical_glyph_metrics(glyph, data.catalogue_identity())
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
fn check(
    m: &VerticalGlyphMetrics<'_>,
    data: &FontData,
    glyph: u32,
    size: f32,
    values: [f64; 3],
    provenance: [P; 3],
) {
    assert!(m.instance().face().is_same_face(&data.face()));
    assert_eq!(
        m.instance().face().catalogue_identity(),
        data.catalogue_identity()
    );
    assert_eq!(m.instance().font_size(), size);
    assert_eq!(m.instance().axis_count(), 0);
    assert_eq!(m.instance().normalized_coords().len(), 0);
    assert_eq!(m.glyph_id(), glyph);
    for (actual, expected) in [m.advance(), m.origin_x(), m.origin_y()]
        .into_iter()
        .zip(values)
    {
        close(actual, expected);
    }
    assert_eq!(
        [
            m.advance_provenance(),
            m.origin_x_provenance(),
            m.origin_y_provenance()
        ],
        provenance
    );
}
fn native_provenance(empty: bool) -> [P; 3] {
    [
        P::VmtxAdvance,
        P::HorizontalAdvanceHalf,
        if empty {
            P::EmptyGlyfTopSideBearing
        } else {
            P::GlyfYMaxTopSideBearing
        },
    ]
}

#[test]
fn unscaled_nominal_facts_retain_exact_integer_and_half_unit_geometry_at_every_size() {
    for (bytes, expected) in [
        (
            fixture::with_native(&fixture::horizontal(800, -200, 0, 1001), 1200, 100, 281),
            [1200.0, 500.5, 900.0],
        ),
        (
            fixture::horizontal(800, -200, 0, 1001),
            [1000.0, 500.5, 800.0],
        ),
    ] {
        x_premises(&bytes, 1001);
        let (system, data) = registered(bytes);
        for size in [f32::from_bits(1), 10.0, 20.0, f32::MAX] {
            let instance = data.unsynthesized_instance(size, &[]).unwrap();
            let raw = instance
                .unscaled_vertical_glyph_metrics(X, system.font_catalogue_identity())
                .unwrap();
            assert_eq!([raw.advance(), raw.origin_x(), raw.origin_y()], expected);
            assert_eq!(raw.units_per_em(), 1000);
            assert_eq!(raw.glyph_id(), X);
            assert!(raw.instance().face().is_same_face(&data.face()));
            assert_eq!(raw.instance().font_size(), size);
            let scaled = instance
                .vertical_glyph_metrics(X, system.font_catalogue_identity())
                .unwrap();
            for (actual, expected) in [scaled.advance(), scaled.origin_x(), scaled.origin_y()]
                .into_iter()
                .zip(expected.map(|v| v * (f64::from(size) / 1000.0)))
            {
                close(actual, expected);
            }
            assert_eq!(scaled.advance_provenance(), raw.advance_provenance());
            assert_eq!(scaled.origin_y_provenance(), raw.origin_y_provenance());
        }
    }
}
fn synthesized(source: L) -> [P; 3] {
    [
        P::HorizontalMetricsHeight(source),
        P::HorizontalAdvanceHalf,
        P::HorizontalMetricsAscent(source),
    ]
}
fn table_error(error: Error, table: FontMetricTable, reason: FontMetricTableError) {
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontMetricTable { table, reason })
    );
}
fn x_premises(bytes: &[u8], width: u16) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    assert_eq!(font.charmap().map(0x58_u32).unwrap().to_u32(), X);
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(
        metrics.advance_width(GlyphId::new(X)),
        Some(f32::from(width))
    );
    let bounds = metrics.bounds(GlyphId::new(X)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
}

fn directory_record(bytes: &[u8], tag: [u8; 4]) -> usize {
    (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|&at| bytes[at..at + 4] == tag)
        .unwrap()
}

#[test]
fn native_metrics_scale_once_and_retain_each_actual_component_provenance() {
    let bytes = fixture::native();
    x_premises(&bytes, 1000);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let row = font.vmtx().unwrap().v_metrics()[58];
    assert_eq!((row.advance.get(), row.side_bearing.get()), (1200, 100));
    let (system, data) = registered(bytes);
    let before = system.stats();
    for (size, expected) in [(10.0, [12.0, 5.0, 9.0]), (20.0, [24.0, 10.0, 18.0])] {
        let m = query(&data, X, size).unwrap();
        check(&m, &data, X, size, expected, native_provenance(false));
    }
    assert_eq!(system.stats(), before);
}

#[test]
fn genuine_absence_uses_a_minus_d_and_ascent_without_ink_or_em_rebasing() {
    for (a, d, gap, expected) in [
        (800, -200, 0, [10.0, 5.0, 8.0]),
        (900, -300, 123, [12.0, 5.0, 9.0]),
        (1000, -200, -100, [12.0, 5.0, 10.0]),
    ] {
        let bytes = fixture::horizontal(a, d, gap, 1000);
        x_premises(&bytes, 1000);
        let font = skrifa::FontRef::new(&bytes).unwrap();
        assert!(font.table_data(skrifa::Tag::new(b"vhea")).is_none());
        assert!(font.table_data(skrifa::Tag::new(b"vmtx")).is_none());
        let hhea = font.hhea().unwrap();
        assert_eq!(
            (
                hhea.ascender().to_i16(),
                hhea.descender().to_i16(),
                hhea.line_gap().to_i16()
            ),
            (a, d, gap)
        );
        let (_, data) = registered(bytes);
        check(
            &query(&data, X, 10.0).unwrap(),
            &data,
            X,
            10.0,
            expected,
            synthesized(L::Hhea),
        );
    }
}

#[test]
fn actual_metric_selector_provenance_and_old_leading_policy_remain_distinct() {
    for use_typo in [false, true] {
        for (hhea_gap, typo_gap) in [(123, 900), (-100, -200)] {
            let bytes =
                crate::baseline_test_fonts::metric_discriminator(use_typo, hhea_gap, typo_gap);
            x_premises(&bytes, 1000);
            let (system, data) = registered(bytes);
            let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
            let before = instance.metrics(system.font_catalogue_identity()).unwrap();
            let (source, a, d, gap) = if use_typo {
                (L::Os2Typographic, 7.0, -2.0, f64::from(typo_gap) / 100.0)
            } else {
                (L::Hhea, 9.0, -3.0, f64::from(hhea_gap) / 100.0)
            };
            assert_eq!(before.line_metric_source(), source);
            close(before.ascent(), a);
            close(before.descent(), d);
            close(before.leading(), gap);
            let snapshot = (
                before.ascent(),
                before.descent(),
                before.leading(),
                before.strikeout(),
            );
            check(
                &query(&data, X, 10.0).unwrap(),
                &data,
                X,
                10.0,
                [a - d, 5.0, a],
                synthesized(source),
            );
            let after = instance.metrics(system.font_catalogue_identity()).unwrap();
            assert_eq!(
                (
                    after.ascent(),
                    after.descent(),
                    after.leading(),
                    after.strikeout()
                ),
                snapshot
            );
        }
    }
    // Real Windows fallback: both earlier typographic sources have zero A/D.
    let base = fixture::absent();
    let mut hhea = table(&base, *b"hhea").to_vec();
    hhea[4..8].fill(0);
    hhea[8..10].copy_from_slice(&(-100_i16).to_be_bytes());
    let mut os2 = table(&base, *b"OS/2").to_vec();
    let flags = u16_at(&os2, 62) & !0x80;
    os2[62..64].copy_from_slice(&flags.to_be_bytes());
    os2[68..72].fill(0);
    os2[74..76].copy_from_slice(&800_u16.to_be_bytes());
    os2[76..78].copy_from_slice(&200_u16.to_be_bytes());
    let (system, data) = registered(replace_tables(
        &base,
        vec![(*b"hhea", hhea), (*b"OS/2", os2)],
    ));
    let metrics = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .metrics(system.font_catalogue_identity())
        .unwrap();
    assert_eq!(metrics.line_metric_source(), L::Os2Windows);
    close(metrics.ascent(), 8.0);
    close(metrics.descent(), -2.0);
    let old_leading = metrics.leading();
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [10.0, 5.0, 8.0],
        synthesized(L::Os2Windows),
    );
    assert_eq!(
        data.unsynthesized_instance(10.0, &[])
            .unwrap()
            .metrics(system.font_catalogue_identity())
            .unwrap()
            .leading(),
        old_leading
    );
}

#[test]
fn odd_horizontal_advance_is_halved_without_rounding_and_size_twenty_doubles() {
    let odd = fixture::horizontal(800, -200, 300, 1001);
    x_premises(&odd, 1001);
    let native = fixture::with_native(&odd, 1200, 100, 281);
    for (bytes, values, provenance) in [
        (odd, [10.0, 5.005, 8.0], synthesized(L::Hhea)),
        (native, [12.0, 5.005, 9.0], native_provenance(false)),
    ] {
        let (_, data) = registered(bytes);
        for size in [10.0, 20.0] {
            check(
                &query(&data, X, size).unwrap(),
                &data,
                X,
                size,
                values.map(|v| v * f64::from(size) / 10.0),
                provenance,
            );
        }
    }
}

#[test]
fn repeated_long_advance_reads_each_glyphs_own_trailing_top_bearing() {
    let bytes = fixture::repeated();
    x_premises(&bytes, 1000);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.vhea().unwrap().number_of_long_ver_metrics(), 1);
    assert_eq!(table(&bytes, *b"vmtx").len(), 564);
    assert_eq!(font.vmtx().unwrap().top_side_bearings()[57].get(), 200);
    let (_, data) = registered(bytes);
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [12.0, 5.0, 10.0],
        native_provenance(false),
    );
    check(
        &query(&data, 3, 10.0).unwrap(),
        &data,
        3,
        10.0,
        [12.0, 5.0, 2.0],
        native_provenance(true),
    );
}

#[test]
fn native_zero_advance_and_negative_origin_remain_valid_signed_facts() {
    let bytes = fixture::zero_signed();
    x_premises(&bytes, 1000);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let row = font.vmtx().unwrap().v_metrics()[58];
    assert_eq!((row.advance.get(), row.side_bearing.get()), (0, -1000));
    let (_, data) = registered(bytes);
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [0.0, 5.0, -2.0],
        native_provenance(false),
    );
}

#[test]
fn genuine_empty_glyf_uses_tsb_natively_but_ascent_when_metrics_are_absent() {
    for (bytes, values, provenance) in [
        (fixture::native(), [12.0, 5.0, 1.0], native_provenance(true)),
        (fixture::absent(), [10.0, 5.0, 8.0], synthesized(L::Hhea)),
    ] {
        assert_eq!(u16_at(table(&bytes, *b"head"), 50), 0);
        let loca = table(&bytes, *b"loca");
        assert_eq!(u16_at(loca, 6), u16_at(loca, 8));
        let font = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(
            font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(3)),
            Some(1000.0)
        );
        let (_, data) = registered(bytes);
        check(
            &query(&data, 3, 10.0).unwrap(),
            &data,
            3,
            10.0,
            values,
            provenance,
        );
    }
}

#[test]
fn valid_composite_headers_are_nominal_references_without_drawing_the_outline() {
    for font in [
        crate::baseline_test_fonts::compound_ahem(),
        crate::baseline_test_fonts::point_attached_ahem(),
    ] {
        let bytes = fixture::with_native(&font, 1200, 100, 281);
        let loca = table(&bytes, *b"loca");
        let at = usize::from(u16_at(loca, 70)) * 2;
        let glyf = table(&bytes, *b"glyf");
        assert_eq!(i16::from_be_bytes(glyf[at..at + 2].try_into().unwrap()), -1);
        assert_eq!(
            i16::from_be_bytes(glyf[at + 8..at + 10].try_into().unwrap()),
            800
        );
        let (_, data) = registered(bytes);
        check(
            &query(&data, 35, 10.0).unwrap(),
            &data,
            35,
            10.0,
            [12.0, 5.0, 9.0],
            native_provenance(false),
        );
    }
}

#[test]
fn required_vertical_pairs_counts_and_lengths_refuse_precisely() {
    for (missing, present) in [
        (*b"vmtx", FontMetricTable::Vhea),
        (*b"vhea", FontMetricTable::Vmtx),
    ] {
        let (_, data) = registered(remove_tables(&fixture::native(), &[missing]));
        let required = if missing == *b"vmtx" {
            FontMetricTable::Vmtx
        } else {
            FontMetricTable::Vhea
        };
        table_error(
            query(&data, X, 10.0).unwrap_err(),
            present,
            FontMetricTableError::MissingCompanion { required },
        );
    }
    for count in [0_u16, 282] {
        let base = fixture::native();
        let mut vhea = table(&base, *b"vhea").to_vec();
        vhea[34..36].copy_from_slice(&count.to_be_bytes());
        let (_, data) = registered(replace_tables(&base, vec![(*b"vhea", vhea)]));
        table_error(
            query(&data, X, 10.0).unwrap_err(),
            FontMetricTable::Vhea,
            FontMetricTableError::LongMetricCount {
                count,
                glyph_count: 281,
            },
        );
    }
    for (tag, table_kind, length) in [
        (*b"maxp", FontMetricTable::Maxp, 5),
        (*b"vhea", FontMetricTable::Vhea, 35),
        (*b"vmtx", FontMetricTable::Vmtx, 1123),
        (
            *b"hmtx",
            FontMetricTable::Hmtx,
            table(&fixture::native(), *b"hmtx").len() - 1,
        ),
    ] {
        let base = fixture::native();
        let mut payload = table(&base, tag).to_vec();
        payload.truncate(length);
        let (_, data) = registered(replace_tables(&base, vec![(tag, payload)]));
        table_error(
            query(&data, X, 10.0).unwrap_err(),
            table_kind,
            FontMetricTableError::Malformed,
        );
    }
    let base = fixture::repeated();
    let mut vmtx = table(&base, *b"vmtx").to_vec();
    vmtx.pop();
    let (_, data) = registered(replace_tables(&base, vec![(*b"vmtx", vmtx)]));
    table_error(
        query(&data, 280, 10.0).unwrap_err(),
        FontMetricTable::Vmtx,
        FontMetricTableError::Malformed,
    );
}

#[test]
fn native_version_and_glyph_request_admission_do_not_clamp_or_synthesize() {
    let base = fixture::native();
    let mut vhea = table(&base, *b"vhea").to_vec();
    vhea[..4].copy_from_slice(&0x0002_0000_u32.to_be_bytes());
    let (_, data) = registered(replace_tables(&base, vec![(*b"vhea", vhea)]));
    table_error(
        query(&data, X, 10.0).unwrap_err(),
        FontMetricTable::Vhea,
        FontMetricTableError::UnsupportedVersion,
    );
    for bytes in [fixture::native(), fixture::absent()] {
        let (_, data) = registered(bytes);
        let error = query(&data, 281, 10.0).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRange);
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::FontGlyphRange {
                glyph: 281,
                count: 281
            })
        );
    }
    let (_, data) = registered(fixture::absent());
    let font = skrifa::FontRef::new(data.bytes()).unwrap();
    assert_eq!(
        font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(0)),
        Some(1000.0)
    );
    check(
        &query(&data, 0, 10.0).unwrap(),
        &data,
        0,
        10.0,
        [10.0, 5.0, 8.0],
        synthesized(L::Hhea),
    );
}

#[test]
fn native_vhea_one_point_one_is_a_supported_version() {
    let base = fixture::native();
    let mut vhea = table(&base, *b"vhea").to_vec();
    vhea[..4].copy_from_slice(&0x0001_1000_u32.to_be_bytes());
    let (_, data) = registered(replace_tables(&base, vec![(*b"vhea", vhea)]));
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [12.0, 5.0, 9.0],
        native_provenance(false),
    );
}

#[test]
fn consumed_vertical_directory_errors_are_not_synthesized_away() {
    let base = fixture::native();
    let record = directory_record(&base, *b"vhea");
    let mut inaccessible = base.clone();
    inaccessible[record + 8..record + 12].copy_from_slice(&u32::MAX.to_be_bytes());
    let mut duplicate = replace_tables(&base, vec![(*b"vheZ", table(&base, *b"vhea").to_vec())]);
    let extra = directory_record(&duplicate, *b"vheZ");
    // Adjacent sorted equal tags, preserving every original required table.
    duplicate[extra..extra + 4].copy_from_slice(b"vhea");
    for (bytes, reason) in [
        (inaccessible, FontMetricTableError::Inaccessible),
        (duplicate, FontMetricTableError::Duplicate),
    ] {
        let (_, data) = registered(bytes);
        table_error(
            query(&data, X, 10.0).unwrap_err(),
            FontMetricTable::Vhea,
            reason,
        );
    }
}

#[test]
fn actual_static_vvar_without_axes_is_an_inconsistent_vertical_source() {
    // Authored valid VVAR1 header and existing real one-axis ItemVariationStore;
    // with a coherent native pair, only the actual fvar axes are missing.
    let mut vvar = vec![0, 1, 0, 0];
    for offset in [24_u32, 0, 0, 0, 0] {
        vvar.extend(offset.to_be_bytes());
    }
    vvar.extend(crate::baseline_test_fonts::one_axis_store(100));
    let (_, data) = registered(replace_tables(
        &fixture::native(),
        vec![(*b"VVAR", vvar.clone())],
    ));
    table_error(
        query(&data, X, 10.0).unwrap_err(),
        FontMetricTable::Vvar,
        FontMetricTableError::VariationWithoutAxes,
    );

    // The orphan also lacks both required vertical companions. The contract
    // requires a missing-companion refusal, not an arbitrary priority between
    // the two absent tags or between unrelated inconsistent-source facts.
    let (_, data) = registered(replace_tables(&fixture::absent(), vec![(*b"VVAR", vvar)]));
    let error = query(&data, X, 10.0).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    let Some(ErrorDetail::FontMetricTable {
        table: FontMetricTable::Vvar,
        reason: FontMetricTableError::MissingCompanion { required },
    }) = error.detail()
    else {
        panic!("expected an actual missing vertical companion, got {error:?}");
    };
    let tag = match required {
        FontMetricTable::Vhea => *b"vhea",
        FontMetricTable::Vmtx => *b"vmtx",
        other => panic!("not a vertical companion: {other:?}"),
    };
    let font = skrifa::FontRef::new(data.bytes()).unwrap();
    assert!(font.table_data(skrifa::Tag::new(&tag)).is_none());
}

#[test]
fn native_required_loca_and_glyf_errors_are_not_empty_but_absence_does_not_read_ink() {
    let base = fixture::native();
    let mut loca = table(&base, *b"loca").to_vec();
    loca.truncate(118);
    let (_, data) = registered(replace_tables(&base, vec![(*b"loca", loca)]));
    table_error(
        query(&data, X, 10.0).unwrap_err(),
        FontMetricTable::Loca,
        FontMetricTableError::Malformed,
    );
    let at = usize::from(u16_at(table(&base, *b"loca"), 116)) * 2;
    let mut glyf = table(&base, *b"glyf").to_vec();
    glyf[at + 4..at + 6].copy_from_slice(&900_i16.to_be_bytes());
    // Stored bbox yMin900 > yMax800 is malformed when actually consumed.
    let bad = replace_tables(&base, vec![(*b"glyf", glyf)]);
    let (_, data) = registered(bad.clone());
    table_error(
        query(&data, X, 10.0).unwrap_err(),
        FontMetricTable::Glyf,
        FontMetricTableError::Malformed,
    );
    let (_, data) = registered(remove_tables(&bad, &[*b"vhea", *b"vmtx"]));
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [10.0, 5.0, 8.0],
        synthesized(L::Hhea),
    );
}

#[test]
fn irrelevant_glyf_vorg_is_ignored_even_if_malformed_or_inaccessible() {
    for native in [false, true] {
        let base = if native {
            fixture::native()
        } else {
            fixture::absent()
        };
        let bad = replace_tables(&base, vec![(*b"VORG", vec![0])]);
        let mut inaccessible = bad.clone();
        let record = (0..usize::from(u16_at(&bad, 4)))
            .map(|i| 12 + i * 16)
            .find(|&at| bad[at..at + 4] == *b"VORG")
            .unwrap();
        inaccessible[record + 8..record + 12].copy_from_slice(&u32::MAX.to_be_bytes());
        for bytes in [bad, inaccessible] {
            let (_, data) = registered(bytes);
            let (values, provenance) = if native {
                ([12.0, 5.0, 9.0], native_provenance(false))
            } else {
                ([10.0, 5.0, 8.0], synthesized(L::Hhea))
            };
            check(
                &query(&data, X, 10.0).unwrap(),
                &data,
                X,
                10.0,
                values,
                provenance,
            );
        }
    }
}

#[test]
fn native_does_not_consume_os2_and_fallback_does_not_consume_unneeded_mvar_or_leading() {
    for (tag, payload) in [(*b"OS/2", vec![0]), (*b"MVAR", vec![0])] {
        let (_, data) = registered(replace_tables(&fixture::native(), vec![(tag, payload)]));
        check(
            &query(&data, X, 10.0).unwrap(),
            &data,
            X,
            10.0,
            [12.0, 5.0, 9.0],
            native_provenance(false),
        );
    }
    let bytes = replace_tables(
        &fixture::horizontal(900, -300, -100, 1000),
        vec![(*b"MVAR", vec![0])],
    );
    let (system, data) = registered(bytes);
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [12.0, 5.0, 9.0],
        synthesized(L::Hhea),
    );
    // The old full metrics operation still requests its legitimate strike
    // variation/leading work and keeps the existing static-MVAR diagnostic.
    table_error(
        data.unsynthesized_instance(10.0, &[])
            .unwrap()
            .metrics(system.font_catalogue_identity())
            .unwrap_err(),
        FontMetricTable::Mvar,
        FontMetricTableError::VariationWithoutAxes,
    );
}

#[test]
fn genuine_variable_default_and_active_instances_refuse_without_static_guessing() {
    let variable = crate::test_fonts::variable_ahem();
    // Build native tables while still static, then retain the already qualified
    // actual fvar/gvar sources. The TrueType-only fixture extrema encoder must
    // not reinterpret a variable provider's empty-outline None as a zero box.
    let native_variable = replace_tables(
        &fixture::native(),
        vec![
            (*b"fvar", table(&variable, *b"fvar").to_vec()),
            (*b"gvar", table(&variable, *b"gvar").to_vec()),
        ],
    );
    for bytes in [variable, native_variable] {
        let (system, data) = registered(bytes);
        for coords in [&[][..], &[0][..], &[16384][..]] {
            let instance = data.unsynthesized_instance(10.0, coords).unwrap();
            assert_eq!(instance.axis_count(), 1);
            let error = instance
                .vertical_glyph_metrics(X, system.font_catalogue_identity())
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::UnsupportedVerticalGlyphVariation {
                    glyph: X,
                    axis_count: 1
                })
            );
        }
    }
}

#[test]
fn genuine_static_cff_is_precisely_unsupported_not_malformed_or_synthesized() {
    let bytes = fixture::cff();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.maxp().unwrap().num_glyphs(), 2);
    assert_eq!(font.charmap().map(0x58_u32).unwrap().to_u32(), 1);
    assert_eq!(
        font.outline_glyphs().format(),
        Some(skrifa::outline::OutlineGlyphFormat::Cff)
    );
    let (_, data) = registered(bytes);
    let error = query(&data, 1, 10.0).unwrap_err();
    assert_eq!(error.code, ErrorCode::FontOutlineRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontOutline {
            glyph: 1,
            reason: FontOutlineError::Unsupported {
                capability: FontOutlineCapability::PostScript
            }
        })
    );
}

#[test]
fn historical_resource_and_expected_publication_keep_exact_request_custody() {
    let (mut system, data) = registered(fixture::native());
    let basis = system.font_catalogue_identity().clone();
    let retained = query(&data, X, 10.0).unwrap();
    system.refresh_fonts().unwrap();
    let fresh = system.registered_font_faces().next().unwrap().to_owned();
    assert!(data.face().is_same_face(&fresh.face()));
    assert_ne!(fresh.catalogue_identity(), &basis);
    let error = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .vertical_glyph_metrics(X, system.font_catalogue_identity())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FontCatalogueMismatch);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontCatalogueMismatch {
            expected: system.font_catalogue_identity().clone(),
            actual: basis.clone()
        })
    );
    let (foreign, _) = registered(fixture::native());
    assert_ne!(foreign.font_catalogue_identity(), &basis);
    assert_eq!(
        data.unsynthesized_instance(10.0, &[])
            .unwrap()
            .vertical_glyph_metrics(X, foreign.font_catalogue_identity())
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    check(
        &query(&fresh, X, 10.0).unwrap(),
        &fresh,
        X,
        10.0,
        [12.0, 5.0, 9.0],
        native_provenance(false),
    );
    drop(system);
    drop(foreign);
    check(
        &retained,
        &data,
        X,
        10.0,
        [12.0, 5.0, 9.0],
        native_provenance(false),
    );
    check(
        &query(&data, X, 10.0).unwrap(),
        &data,
        X,
        10.0,
        [12.0, 5.0, 9.0],
        native_provenance(false),
    );
    for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(
            data.unsynthesized_instance(size, &[]).unwrap_err().code,
            ErrorCode::InvalidFontInstance
        );
    }
    assert_eq!(
        data.unsynthesized_instance(10.0, &[1])
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::FontCoordinateCount {
            expected: 0,
            actual: 1
        })
    );
}

#[test]
fn querying_actual_run_instance_leaves_source_glyphs_carets_and_cache_unchanged() {
    let (mut system, _) = registered(fixture::native());
    let source = Source::identified("X", Id::from_u64(1513001), 7);
    let analysis = AnalyzedSource::new(
        source.clone(),
        Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            ..Style::default()
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let options = Options {
        quantize: false,
        ..Options::default()
    };
    let old = system.layout_analyzed(&analysis, options).unwrap();
    let glyphs = old
        .glyph_runs()
        .iter()
        .flat_map(|r| r.glyphs())
        .map(|g| (g.id(), g.advance(), g.range()))
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(58, 10.0, Range::new(0, 1))]);
    let positions = old.positions().collect::<Vec<_>>();
    let rects = positions
        .iter()
        .map(|p| old.position_geometry(p).unwrap().rect())
        .collect::<Vec<_>>();
    let stats = system.stats();
    let run = old.run_views().next().unwrap();
    let instance = run.effective_instance().unwrap();
    let m = instance
        .vertical_glyph_metrics(58, system.font_catalogue_identity())
        .unwrap();
    close(m.advance(), 12.0);
    close(m.origin_x(), 5.0);
    close(m.origin_y(), 9.0);
    assert!(m.instance().face().is_same_face(&run.face()));
    assert_eq!(system.stats(), stats);
    assert_eq!(old.source(), &source);
    assert_eq!(
        old.glyph_runs()
            .iter()
            .flat_map(|r| r.glyphs())
            .map(|g| (g.id(), g.advance(), g.range()))
            .collect::<Vec<_>>(),
        glyphs
    );
    assert_eq!(
        positions
            .iter()
            .map(|p| old.position_geometry(p).unwrap().rect())
            .collect::<Vec<_>>(),
        rects
    );
    let warm = system.layout_analyzed(&analysis, options).unwrap();
    assert_eq!(warm.key(), old.key());
    assert_eq!(system.stats().layout_hits(), stats.layout_hits() + 1);
}

#[test]
fn actual_selected_faux_synthesis_is_retained_but_does_not_adjust_nominal_metrics() {
    let (mut system, data) = registered(fixture::native());
    let layout = system
        .layout(
            Source::new("X"),
            Style {
                font: Font::new()
                    .family("Ahem")
                    .weight(Weight::Bold)
                    .style(Slant::Italic),
                size: 10.0,
                ..Style::default()
            },
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap();
    let run = layout.run_views().next().unwrap();
    let instance = run.effective_instance().unwrap();
    assert!(instance.embolden() > 0.0, "real selected bold synthesis");
    assert!(instance.x_shear() > 0.0, "real selected italic synthesis");
    assert!(instance.face().is_same_face(&data.face()));
    let observation = instance
        .vertical_glyph_metrics(X, system.font_catalogue_identity())
        .unwrap();
    check(
        &observation,
        &data,
        X,
        10.0,
        [12.0, 5.0, 9.0],
        native_provenance(false),
    );
    assert_eq!(observation.instance().synthesis(), instance.synthesis());
    assert_eq!(
        layout
            .glyph_runs()
            .iter()
            .flat_map(|r| r.glyphs())
            .map(|g| (g.id(), g.advance(), g.range()))
            .collect::<Vec<_>>(),
        [(58, 10.0, Range::new(0, 1))]
    );
}

#[test]
fn negative_contour_tag_minus_two_keeps_a_real_composite_reference() {
    use skrifa::outline::{DrawSettings, pen::ControlBoundsPen};
    let original = crate::baseline_test_fonts::compound_ahem();
    assert_eq!(u16_at(table(&original, *b"head"), 50), 0);
    let at = usize::from(u16_at(table(&original, *b"loca"), 70)) * 2;
    let mut glyf = table(&original, *b"glyf").to_vec();
    assert_eq!(i16::from_be_bytes(glyf[at..at + 2].try_into().unwrap()), -1);
    // OpenType glyf: every negative contour count denotes a composite. Only
    // the recommended spelling changes; actual component flags/ID/XY/bbox stay.
    glyf[at..at + 2].copy_from_slice(&(-2_i16).to_be_bytes());
    let font = replace_tables(&original, vec![(*b"glyf", glyf)]);
    let bytes = fixture::with_native(&font, 1200, 100, 281);
    let raw = table(&bytes, *b"glyf");
    assert_eq!(i16::from_be_bytes(raw[at..at + 2].try_into().unwrap()), -2);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map(0x41_u32).unwrap().to_u32(), 35);
    let outlines = face.outline_glyphs();
    let glyph = outlines.get(GlyphId::new(35)).unwrap();
    let mut pen = ControlBoundsPen::default();
    glyph
        .draw(
            DrawSettings::unhinted(FontSize::unscaled(), LocationRef::default()),
            &mut pen,
        )
        .unwrap();
    let bounds = pen.bounding_box().unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    let (_, data) = registered(bytes);
    check(
        &query(&data, 35, 10.0).unwrap(),
        &data,
        35,
        10.0,
        [12.0, 5.0, 9.0],
        native_provenance(false),
    );
}
