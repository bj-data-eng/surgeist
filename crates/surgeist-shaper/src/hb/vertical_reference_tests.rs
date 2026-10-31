//! Actual pre-positioning reference semantics, not a post-GPOS origin repair.
use super::*;
use crate::{VerticalOriginReference as Reference, VerticalOriginReferenceError as ReferenceError};

fn shape(
    font: &Shaper<'_>,
    text: &str,
    features: &[Feature],
    reference: Reference,
) -> VerticalGlyphBuffer {
    let source = input(text, Direction::TopToBottom);
    let plan = make_plan(font, &source, features);
    font.shape_vertical_with_metrics(&plan, source, features, reference, |gid| {
        Ok::<_, Infallible>(font::metrics(font, gid))
    })
    .unwrap()
}
fn absolute(value: f64) -> Reference {
    Reference::try_font_design_x(value).unwrap()
}
fn relative(value: f64) -> Reference {
    Reference::try_nominal_vertical_pen_x(value).unwrap()
}

// Re-encode this actual authored font with only the requested table replaced.
// No duplicate tag, installed resource, provider output or invented metric ID.
fn replace_table(
    font: &Shaper<'_>,
    replacement: [u8; 4],
    data: &[u8],
    f: impl FnOnce(&Shaper<'_>),
) {
    let mut tables = font
        .font
        .table_directory
        .table_records()
        .iter()
        .map(|row| {
            let tag = row.tag().to_be_bytes();
            let bytes = if tag == replacement {
                data.to_vec()
            } else {
                font.font.table_data(row.tag()).unwrap().as_bytes().to_vec()
            };
            (tag, bytes)
        })
        .collect::<Vec<_>>();
    tables.sort_by_key(|row| row.0);
    let mut bytes = vec![0; 12 + tables.len() * 16];
    bytes[..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    bytes[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    for (i, (tag, data)) in tables.iter().enumerate() {
        let record = 12 + 16 * i;
        bytes[record..record + 4].copy_from_slice(tag);
        let start = u32::try_from(bytes.len()).unwrap();
        bytes[record + 8..record + 12].copy_from_slice(&start.to_be_bytes());
        bytes[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend(data);
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
    }
    let face = crate::FontRef::new(&bytes).unwrap();
    let shared = crate::ShaperData::new(&face);
    let actual = shared.shaper(&face).build();
    f(&actual);
}
fn with_different_widths(gsub: Option<&[u8]>, gpos: Option<&[u8]>, f: impl FnOnce(&Shaper<'_>)) {
    font::with_font(true, true, gsub, gpos, &[], |font| {
        let mut horizontal = font
            .font
            .table_data(crate::Tag::new(b"hmtx"))
            .unwrap()
            .as_bytes()
            .to_vec();
        let at = usize::from(font::B) * 4;
        horizontal[at..at + 2].copy_from_slice(&800u16.to_be_bytes());
        replace_table(font, *b"hmtx", &horizontal, |font| {
            assert_eq!(font::metrics(font, 35).origin_x_twice(), 1001);
            assert_eq!(font::metrics(font, 36).origin_x_twice(), 800);
            f(font);
        });
    });
}

#[test]
fn reference_constructors_keep_half_units_endpoints_signed_zero_and_precise_failure_order() {
    for value in [-1_073_741_824.0, -600.5, -0.0, 0.0, 500.5, 1_073_741_823.5] {
        let twice = (value * 2.0) as i32;
        assert_eq!(
            absolute(value),
            Reference::FontDesignX {
                coordinate_twice: twice
            }
        );
        assert_eq!(
            relative(value),
            Reference::NominalVerticalPenX {
                offset_twice: twice
            }
        );
    }
    for (value, expected) in [
        (f64::NAN, ReferenceError::NonFinite),
        (f64::INFINITY, ReferenceError::NonFinite),
        (f64::NEG_INFINITY, ReferenceError::NonFinite),
        (1_073_741_824.0, ReferenceError::OutOfRange),
        (-1_073_741_824.5, ReferenceError::OutOfRange),
        (1_073_741_823.75, ReferenceError::OutOfRange),
        (500.375, ReferenceError::NotHalfUnit),
        (-0.25, ReferenceError::NotHalfUnit),
    ] {
        assert_eq!(Reference::try_font_design_x(value), Err(expected));
        assert_eq!(Reference::try_nominal_vertical_pen_x(value), Err(expected));
    }
}

#[test]
fn absolute_origin_is_font_wide_while_relative_distance_adds_each_actual_glyph_width() {
    with_different_widths(None, None, |font| {
        let absolute = shape(font, "AB", &[], absolute(600.0));
        assert_eq!(ids(&absolute), [(35, 0), (36, 1)]);
        assert_eq!(
            positions(&absolute),
            [
                (0.0, -1200.0, -600.0, -900.0),
                (0.0, -1400.0, -600.0, -1000.0)
            ]
        );
        let relative = shape(font, "AB", &[], relative(200.0));
        assert_eq!(
            positions(&relative),
            [
                (0.0, -1200.0, -700.5, -900.0),
                (0.0, -1400.0, -600.0, -1000.0)
            ]
        );
        assert_eq!(absolute.shaping_status(), ShapingStatus::Complete);
        assert_eq!(relative.shaping_status(), ShapingStatus::Complete);
    });
}

#[test]
fn actual_gsub_replacement_metrics_are_nominal_and_reference_is_applied_after_substitution() {
    let gsub = font::layout_table(b"ccmp", &[font::replace()]);
    font::with_font(true, true, Some(&gsub), None, &[], |font| {
        let control = ordinary(font, "A", Direction::TopToBottom, &[]);
        assert_eq!(control.glyph_infos()[0].glyph_id, 58);
        for (reference, expected) in [(absolute(600.0), -600.0), (relative(200.0), -800.5)] {
            let source = input("A", Direction::TopToBottom);
            let plan = make_plan(font, &source, &[]);
            let mut asked = Vec::new();
            let result = font
                .shape_vertical_with_metrics(&plan, source, &[], reference, |gid| {
                    asked.push(gid);
                    Ok::<_, Infallible>(font::metrics(font, gid))
                })
                .unwrap();
            assert!(!asked.is_empty());
            assert!(asked.iter().all(|&gid| gid == 58));
            assert_eq!(ids(&result), [(58, 0)]);
            assert_eq!(positions(&result), [(0.0, -1600.0, expected, -1000.0)]);
        }
    });
}

#[test]
fn real_pair_and_single_gpos_consume_effective_seed_before_integer_adjustments() {
    let features = feature(b"vkrn");
    for lookup in [font::single(), font::pair()] {
        let is_pair = lookup.0 == 2;
        let gpos = font::layout_table(b"vkrn", &[lookup]);
        with_different_widths(None, Some(&gpos), |font| {
            let text = if is_pair { "AB" } else { "A" };
            for (reference, x) in [(absolute(600.0), -570.0), (relative(200.0), -670.5)] {
                let result = shape(font, text, &features, reference);
                let mut expected = vec![(0.0, -1250.0, x, -860.0)];
                if is_pair {
                    expected.push((0.0, -1390.0, -620.0, -1030.0));
                }
                assert_eq!(positions(&result), expected);
            }
        });
    }
}

#[test]
fn real_mark_attachment_carries_effective_parent_reference_once_not_child_width_repair() {
    let gpos = font::layout_table(
        b"mark",
        &[
            font::mark(font::A, font::MARK, (300, 700), (20, 100), 4),
            font::mark(font::MARK, font::MARK2, (70, 400), (10, 50), 6),
        ],
    );
    with_different_widths(None, Some(&gpos), |font| {
        let features = feature(b"mark");
        for (reference, base_x, first, second) in [
            (absolute(600.0), -600.0, -320.0, -260.0),
            (relative(200.0), -700.5, -420.5, -360.5),
        ] {
            let result = shape(font, "A\u{300}\u{315}", &features, reference);
            assert_eq!(
                positions(&result),
                [
                    (0.0, -1200.0, base_x, -900.0),
                    (0.0, 0.0, first, 900.0),
                    (0.0, 0.0, second, 1250.0)
                ]
            );
            assert!(result.glyph_infos()[1..].iter().all(|g| g.is_mark()));
        }
        // Different enum references with equal actual effective parent origins
        // must agree, despite unrelated nominal widths on the overwritten marks.
        assert_eq!(
            positions(&shape(font, "A\u{300}", &features, absolute(500.5))),
            positions(&shape(font, "A\u{300}", &features, relative(-0.0)))
        );
    });
}

#[test]
fn genuine_vertical_cursive_propagates_effective_parent_cross_origin_without_post_repair() {
    let gpos = font::layout_table(b"curs", &[font::cursive()]);
    with_different_widths(None, Some(&gpos), |font| {
        let features = feature(b"curs");
        assert_eq!(
            positions(&shape(font, "AB", &features, absolute(600.0))),
            [(0.0, -400.0, -600.0, -900.0), (0.0, -600.0, -400.0, -200.0)]
        );
        assert_eq!(
            positions(&shape(font, "AB", &features, relative(200.0))),
            [(0.0, -400.0, -700.5, -900.0), (0.0, -600.0, -500.5, -200.0)]
        );
    });
}

#[test]
fn separate_no_gpos_fallback_keeps_raw_quotient_and_actual_effective_base_origin() {
    font::with_font(true, true, None, None, &[], |font| {
        assert!(font.font.gpos().is_err());
        let mut glyf = font
            .font
            .table_data(crate::Tag::new(b"glyf"))
            .unwrap()
            .as_bytes()
            .to_vec();
        let loca = font.font.table_data(crate::Tag::new(b"loca")).unwrap();
        let at = usize::from(font::MARK) * 4;
        let start = u32::from_be_bytes(loca.as_bytes()[at..at + 4].try_into().unwrap()) as usize;
        for (offset, value) in [(6, 958i16), (20, 958), (24, -958)] {
            glyf[start + offset..start + offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        replace_table(font, *b"glyf", &glyf, |font| {
            let raw = font.font.glyf().unwrap();
            let locations = font.font.loca(Some(true)).unwrap();
            let mark = locations
                .get_glyf(read_fonts::types::GlyphId::new(100), &raw)
                .unwrap()
                .unwrap();
            assert_eq!((mark.x_min(), mark.x_max()), (0, 958));
            let old = ordinary(font, "A\u{300}", Direction::TopToBottom, &[]);
            assert_eq!(old_positions(&old)[1].2, -479); //21−integerOrigin500.
            for (reference, expected) in [
                (relative(200.0), -679.5),
                (absolute(600.0), -579.0),
                (relative(0.0), -479.5),
                (absolute(500.5), -479.5),
            ] {
                let result = shape(font, "A\u{300}", &[], reference);
                // trunc((1001−958)/2)=21, then the actual effective origin once.
                assert_eq!(positions(&result)[1], (0.0, 0.0, expected, 1181.0));
            }
        });
    });
}

#[test]
fn representable_references_may_fail_only_when_actual_glyph_combination_overflows() {
    font::with_font(true, true, None, None, &[], |font| {
        for reference in [relative(1_073_741_823.5), absolute(-1_073_741_824.0)] {
            let source = input("A", Direction::TopToBottom);
            let plan = make_plan(font, &source, &[]);
            let error =
                failure(
                    font.shape_vertical_with_metrics(&plan, source, &[], reference, |gid| {
                        Ok::<_, Infallible>(font::metrics(font, gid))
                    }),
                );
            assert!(matches!(
                error,
                VerticalShapeError::CoordinateRange {
                    stage: VerticalPositionStage::Nominal
                }
            ));
        }
        let source = input("AB", Direction::TopToBottom);
        let plan = make_plan(font, &source, &[]);
        let error =
            failure(
                font.shape_vertical_with_metrics(&plan, source, &[], absolute(600.0), |gid| {
                    if gid == 36 {
                        Err("retained later metric error")
                    } else {
                        Ok(font::metrics(font, gid))
                    }
                }),
            );
        assert!(matches!(
            error,
            VerticalShapeError::Supplier {
                glyph_id: 36,
                error: "retained later metric error"
            }
        ));
        let mut source = input("A", Direction::TopToBottom);
        source.0.max_len = 1;
        assert!(!source.reserve(2));
        let plan = make_plan(font, &source, &[]);
        let mut called = false;
        let error = failure(font.shape_vertical_with_metrics(
            &plan,
            source,
            &[],
            relative(1_073_741_823.5),
            |_| {
                called = true;
                Err::<VerticalNominalMetrics, _>("would fail")
            },
        ));
        assert!(!called);
        assert!(matches!(
            error,
            VerticalShapeError::Refused(ShapingRefusal::GrowthLimit)
        ));
    });
}

#[test]
fn references_are_invocation_local_and_di_clear_restores_ordinary_pen_semantics() {
    font::with_font(true, true, None, None, &[], |font| {
        let reference = absolute(600.0);
        let result = shape(font, "A\u{2065}", &[], reference);
        assert_eq!(ids(&result), [(35, 0), (274, 1)]);
        let receipt = result.default_ignorable_receipt().unwrap();
        assert_eq!(receipt.residual_input_positions(), [1]);
        assert_eq!(receipt.residual_glyph_positions(), [1]);
        assert_eq!(positions(&result)[0], (0.0, -1200.0, -600.0, -900.0));
        let mut cleared = result.clear();
        cleared.add('A', 13);
        cleared.set_direction(Direction::TopToBottom);
        cleared.set_script(script::LATIN);
        let old = font.shape(cleared, &[]);
        assert_eq!(old_positions(&old), [(0, -1200, -500, -900)]);
        assert_eq!(old.glyph_infos()[0].cluster, 13);
        // Reuse the actual same plan while references change across invocations.
        let fresh = input("A", Direction::TopToBottom);
        let plan = make_plan(font, &fresh, &[]);
        for (reference, x) in [
            (relative(-0.0), -500.5),
            (relative(200.0), -700.5),
            (absolute(600.0), -600.0),
            (relative(0.0), -500.5),
        ] {
            let result = font
                .shape_vertical_with_metrics(
                    &plan,
                    input("A", Direction::TopToBottom),
                    &[],
                    reference,
                    |gid| Ok::<_, Infallible>(font::metrics(font, gid)),
                )
                .unwrap();
            assert_eq!(positions(&result), [(0.0, -1200.0, x, -900.0)]);
        }
    });
}
