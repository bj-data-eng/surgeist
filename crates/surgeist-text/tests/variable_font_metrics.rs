//! Glyph-free same-instance metrics over registered derivatives of licensed Ahem.
//! Literal OpenType MVAR field associations and Item Variation Store interpolation
//! distinguish static selection, actual coordinates, row completeness and rounding.
//! LONG_WORDS is deliberate supported format admission, not a COLR execution test.
#[path = "../src/test_fonts.rs"]
mod test_fonts;

use surgeist_text::{
    ErrorCode, ErrorDetail, FontCatalogueMode, FontData, FontMetricTable, System, SystemOptions,
};
use test_fonts::{
    remove_tables, replace_tables, table, two_face_collection, u16_at, variable_ahem,
    variable_ahem_with_tuple,
};

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const ACTIVE: (i16, i16, i16) = (0, 16384, 16384);

fn system() -> System {
    System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap()
}
fn put16(bytes: &mut [u8], at: usize, value: i16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}
fn put_u16(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}
fn put32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

// Binary fixture encoder only: no metric/interpolation implementation or oracle.
// Every row uses all region indices in order, with no shared data subtables.
fn mvar(
    tags: &[[u8; 4]],
    regions: &[Vec<(i16, i16, i16)>],
    rows: &[Vec<i32>],
    long: bool,
) -> Vec<u8> {
    assert_eq!(tags.len(), rows.len());
    assert!(tags.windows(2).all(|pair| pair[0] < pair[1]));
    let axes = regions[0].len();
    assert!(regions.iter().all(|region| region.len() == axes));
    assert!(rows.iter().all(|row| row.len() == regions.len()));
    let store_at = 12 + tags.len() * 8;
    let data_at = 12 + 4 + regions.len() * axes * 6;
    let mut output = Vec::new();
    for value in [
        1,
        0,
        0,
        8,
        u16::try_from(tags.len()).unwrap(),
        u16::try_from(store_at).unwrap(),
    ] {
        output.extend_from_slice(&value.to_be_bytes());
    }
    for (row, tag) in tags.iter().enumerate() {
        output.extend_from_slice(tag);
        output.extend_from_slice(&0u16.to_be_bytes());
        output.extend_from_slice(&u16::try_from(row).unwrap().to_be_bytes());
    }
    output.extend_from_slice(&1u16.to_be_bytes()); // ItemVariationStore format
    output.extend_from_slice(&12u32.to_be_bytes()); // store-relative region list
    output.extend_from_slice(&1u16.to_be_bytes());
    output.extend_from_slice(&u32::try_from(data_at).unwrap().to_be_bytes());
    output.extend_from_slice(&u16::try_from(axes).unwrap().to_be_bytes());
    output.extend_from_slice(&u16::try_from(regions.len()).unwrap().to_be_bytes());
    for region in regions {
        for &(start, peak, end) in region {
            for value in [start, peak, end] {
                output.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
    output.extend_from_slice(&u16::try_from(rows.len()).unwrap().to_be_bytes());
    let words = u16::try_from(regions.len()).unwrap() | if long { 0x8000 } else { 0 };
    output.extend_from_slice(&words.to_be_bytes());
    output.extend_from_slice(&u16::try_from(regions.len()).unwrap().to_be_bytes());
    for index in 0..regions.len() {
        output.extend_from_slice(&u16::try_from(index).unwrap().to_be_bytes());
    }
    for row in rows {
        for &delta in row {
            if long {
                output.extend_from_slice(&delta.to_be_bytes());
            } else {
                output.extend_from_slice(&i16::try_from(delta).unwrap().to_be_bytes());
            }
        }
    }
    output
}
fn primary() -> Vec<u8> {
    let bytes = mvar(
        &[*b"hasc", *b"hdsc", *b"hlgp", *b"stro", *b"strs"],
        &[vec![ACTIVE]],
        &[vec![100], vec![-50], vec![20], vec![40], vec![10]],
        false,
    );
    assert_eq!(bytes.len(), 92);
    assert_eq!(u16_at(&bytes, 10), 52);
    bytes
}
fn with_mvar(font: &[u8], data: Vec<u8>) -> Vec<u8> {
    replace_tables(font, vec![(*b"MVAR", data)])
}
fn typo(font: &[u8]) -> Vec<u8> {
    let mut os2 = table(font, *b"OS/2").to_vec();
    // Bit7 is assigned starting in OS/2 version4; original Ahem is version3.
    put_u16(&mut os2, 0, 4);
    let selection = u16_at(&os2, 62) | 0x80;
    put_u16(&mut os2, 62, selection);
    replace_tables(font, vec![(*b"OS/2", os2)])
}
fn line_fields(
    font: &[u8],
    hhea_values: [i16; 3],
    typo_values: [i16; 3],
    win: [u16; 2],
) -> Vec<u8> {
    let mut hhea = table(font, *b"hhea").to_vec();
    let mut os2 = table(font, *b"OS/2").to_vec();
    for (at, value) in [4, 6, 8].into_iter().zip(hhea_values) {
        put16(&mut hhea, at, value);
    }
    for (at, value) in [68, 70, 72].into_iter().zip(typo_values) {
        put16(&mut os2, at, value);
    }
    put_u16(&mut os2, 74, win[0]);
    put_u16(&mut os2, 76, win[1]);
    replace_tables(font, vec![(*b"hhea", hhea), (*b"OS/2", os2)])
}
fn two_axis_font() -> Vec<u8> {
    // No glyph tuples, so the actual gvar axis inventory can be changed without
    // leaving a one-axis tuple under a two-axis fvar declaration.
    let base = variable_ahem_with_tuple(&[]);
    let mut fvar = table(&base, *b"fvar").to_vec();
    let mut second = fvar[16..36].to_vec();
    second[..4].copy_from_slice(b"wdth");
    put_u16(&mut second, 18, 257);
    fvar.extend(second);
    put_u16(&mut fvar, 8, 2);
    put_u16(&mut fvar, 14, 12); // zero named instances still require coherent record size
    let mut gvar = table(&base, *b"gvar").to_vec();
    put_u16(&mut gvar, 4, 2);
    replace_tables(&base, vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}

#[derive(Clone, Copy)]
struct Query<'a> {
    size: f32,
    coords: &'a [i16],
    complete_coords: &'a [i16],
    // Scaled ascent, descent, leading, strike center, strike thickness.
    expected: [f64; 5],
}
fn close(label: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(f64::MIN_POSITIVE),
        "{label}: {actual} != {expected}"
    );
}
fn query(label: &str, actual: &System, data: &FontData, q: Query<'_>, failures: &mut Vec<String>) {
    let stats = actual.stats();
    let instance = data.unsynthesized_instance(q.size, q.coords).unwrap();
    assert_eq!(instance.axis_count(), q.complete_coords.len(), "{label}");
    assert_eq!(
        instance.normalized_coords().collect::<Vec<_>>(),
        q.complete_coords,
        "{label}"
    );
    let outcome = instance.metrics(actual.font_catalogue_identity());
    assert_eq!(actual.stats(), stats, "{label}: metrics must not publish");
    match outcome {
        Err(error) => {
            eprintln!("{label} coords={:?}: {error:?}", q.coords);
            failures.push(format!("{label} {:?}: {error:?}", q.coords));
        }
        Ok(metrics) => {
            assert_eq!(metrics.units_per_em(), 1000);
            assert_eq!(metrics.instance().font_size(), q.size);
            assert_eq!(
                metrics.instance().normalized_coords().collect::<Vec<_>>(),
                q.complete_coords
            );
            assert!(metrics.instance().face().is_same_face(&data.face()));
            assert_eq!(
                metrics.instance().face().bytes().as_ptr(),
                data.bytes().as_ptr()
            );
            for (value, expected) in [
                metrics.ascent(),
                metrics.descent(),
                metrics.leading(),
                metrics.strikeout().center_offset(),
                metrics.strikeout().thickness(),
            ]
            .into_iter()
            .zip(q.expected)
            {
                close(label, value, expected);
            }
            assert!(metrics.ascent() >= 0.0 && metrics.strikeout().thickness() > 0.0);
        }
    }
}
fn run_font(label: &str, bytes: Vec<u8>, queries: &[Query<'_>], failures: &mut Vec<String>) {
    let mut actual = system();
    if let Err(error) = actual.register_fonts(bytes.clone()) {
        failures.push(format!(
            "{label}: registration refused before metrics: {error:?}"
        ));
        return;
    }
    let data = actual.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(data.index(), 0);
    for &q in queries {
        query(label, &actual, &data, q, failures);
    }
    drop(actual);
    assert_eq!(
        data.bytes(),
        bytes,
        "{label}: retained bytes outlive System"
    );
}
fn finish(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "same-instance metrics must execute:\n{}",
        failures.join("\n")
    );
}

#[test]
fn variable_without_mvar_and_primary_typo_metrics_use_actual_complete_coordinates() {
    let mut failures = Vec::new();
    run_font(
        "real gvar without MVAR",
        variable_ahem(),
        &[
            Query {
                size: 20.0,
                coords: &[],
                complete_coords: &[0],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            },
            Query {
                size: 20.0,
                coords: &[8192],
                complete_coords: &[8192],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            },
            Query {
                size: 20.0,
                coords: &[16384],
                complete_coords: &[16384],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            },
        ],
        &mut failures,
    );
    run_font(
        "92-byte primary MVAR with version4 typo selection",
        with_mvar(&typo(&variable_ahem()), primary()),
        &[
            Query {
                size: 20.0,
                coords: &[],
                complete_coords: &[0],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            },
            Query {
                size: 20.0,
                coords: &[0],
                complete_coords: &[0],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            },
            Query {
                size: 20.0,
                coords: &[8192],
                complete_coords: &[8192],
                expected: [17.0, -4.5, 0.2, 5.03, 1.1],
            },
            Query {
                size: 20.0,
                coords: &[16384],
                complete_coords: &[16384],
                expected: [18.0, -5.0, 0.4, 5.38, 1.2],
            },
        ],
        &mut failures,
    );
    run_font(
        "original non-typo hhea ignores line tags but varies strike",
        with_mvar(&variable_ahem(), primary()),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [16.0, -4.0, 0.0, 5.38, 1.2],
        }],
        &mut failures,
    );
    finish(failures);
}

#[test]
fn line_deltas_follow_original_selected_fields_and_windows_descent_stored_sign() {
    let mut failures = Vec::new();
    let distinct = line_fields(
        &variable_ahem(),
        [1000, -300, 7],
        [800, -200, 11],
        [900, 300],
    );
    for (label, font, expected) in [
        (
            "explicit typo",
            typo(&distinct),
            [18.0, -5.0, 0.62, 5.38, 1.2],
        ),
        (
            "nonzero hhea",
            distinct.clone(),
            [20.0, -6.0, 0.14, 5.38, 1.2],
        ),
        (
            "fallback typo",
            line_fields(&distinct, [0, 0, 7], [800, -200, 11], [900, 300]),
            [18.0, -5.0, 0.62, 5.38, 1.2],
        ),
    ] {
        run_font(
            label,
            with_mvar(&font, primary()),
            &[Query {
                size: 20.0,
                coords: &[16384],
                complete_coords: &[16384],
                expected,
            }],
            &mut failures,
        );
    }
    let windows = line_fields(&variable_ahem(), [0, 0, 7], [0, 0, 99], [900, 300]);
    let table = mvar(
        &[*b"hasc", *b"hcla", *b"hcld", *b"hdsc", *b"hlgp"],
        &[vec![ACTIVE]],
        &[vec![777], vec![100], vec![50], vec![-777], vec![20]],
        false,
    );
    for (label, font, gap) in [
        ("Windows retains hhea gap", windows.clone(), 0.14),
        (
            "Windows without hhea initializes gap zero",
            remove_tables(&windows, &[*b"hhea"]),
            0.0,
        ),
    ] {
        run_font(
            label,
            with_mvar(&font, table.clone()),
            &[Query {
                size: 20.0,
                coords: &[16384],
                complete_coords: &[16384],
                expected: [20.0, -7.0, gap, 4.68, 1.0],
            }],
            &mut failures,
        );
    }
    // USE_TYPO zero values stay selected; variation must not switch to Windows.
    let zero = typo(&line_fields(
        &variable_ahem(),
        [1000, -300, 7],
        [0, 0, 11],
        [900, 300],
    ));
    run_font(
        "zero typo remains selected",
        with_mvar(&zero, primary()),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [2.0, -1.0, 0.62, 5.38, 1.2],
        }],
        &mut failures,
    );
    finish(failures);
}

#[test]
fn null_slots_sentinel_and_zero_record_tables_are_legitimate_zero_adjustments() {
    let mut failures = Vec::new();
    for inner in [7, u16::MAX] {
        let mut data = primary();
        put32(&mut data, 60, 0); // actual declared data slot is null
        for at in [18, 26, 34, 42, 50] {
            put_u16(&mut data, at, inner);
        }
        run_font(
            &format!("null slot ignores inner {inner}"),
            with_mvar(&typo(&variable_ahem()), data),
            &[Query {
                size: 20.0,
                coords: &[16384],
                complete_coords: &[16384],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            }],
            &mut failures,
        );
    }
    let mut data = primary();
    for at in [16, 24, 32, 40, 48] {
        put_u16(&mut data, at, u16::MAX);
        put_u16(&mut data, at + 2, u16::MAX);
    }
    run_font(
        "exact no-variation sentinel",
        with_mvar(&typo(&variable_ahem()), data),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [16.0, -4.0, 0.0, 4.68, 1.0],
        }],
        &mut failures,
    );
    let mut empty = primary()[..12].to_vec();
    put_u16(&mut empty, 8, 0);
    put_u16(&mut empty, 10, 0);
    run_font(
        "zero records and null store",
        with_mvar(&typo(&variable_ahem()), empty),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [16.0, -4.0, 0.0, 4.68, 1.0],
        }],
        &mut failures,
    );
    finish(failures);
}

#[test]
fn neutral_regions_execute_at_default_and_two_axis_order_is_not_erased() {
    let mut failures = Vec::new();
    for (label, neutral) in [
        ("peak zero", (-16384, 0, 16384)),
        ("misordered start", (16384, 8192, 16384)),
        ("crossing zero", (-16384, 8192, 16384)),
    ] {
        let data = mvar(&[*b"hasc"], &[vec![neutral]], &[vec![100]], false);
        run_font(
            label,
            with_mvar(&typo(&variable_ahem()), data),
            &[
                Query {
                    size: 1000.0,
                    coords: &[],
                    complete_coords: &[0],
                    expected: [900.0, -200.0, 0.0, 234.0, 50.0],
                },
                Query {
                    size: 1000.0,
                    coords: &[16384],
                    complete_coords: &[16384],
                    expected: [900.0, -200.0, 0.0, 234.0, 50.0],
                },
            ],
            &mut failures,
        );
        let data = mvar(&[*b"hasc"], &[vec![neutral, ACTIVE]], &[vec![100]], false);
        run_font(
            &format!("two axes {label}"),
            with_mvar(&typo(&two_axis_font()), data),
            &[
                Query {
                    size: 1000.0,
                    coords: &[],
                    complete_coords: &[0, 0],
                    expected: [800.0, -200.0, 0.0, 234.0, 50.0],
                },
                Query {
                    size: 1000.0,
                    coords: &[0, 8192],
                    complete_coords: &[0, 8192],
                    expected: [850.0, -200.0, 0.0, 234.0, 50.0],
                },
                Query {
                    size: 1000.0,
                    coords: &[8192, 0],
                    complete_coords: &[8192, 0],
                    expected: [800.0, -200.0, 0.0, 234.0, 50.0],
                },
            ],
            &mut failures,
        );
    }
    finish(failures);
}

#[test]
fn metric_rows_round_once_with_positive_infinity_ties_and_wide_accumulation() {
    let mut failures = Vec::new();
    let rows = [
        ("two positive halves", vec![1, 1], false, 8192, 801.0),
        ("two negative halves", vec![-1, -1], false, 8192, 799.0),
        ("positive half tie", vec![1], false, 8192, 801.0),
        ("negative half tie", vec![-1], false, 8192, 800.0),
        (
            "normal16 aggregate exceeds fixed integer domain",
            vec![20000, 20000],
            false,
            16384,
            40800.0,
        ),
        ("deliberate long32 delta", vec![40000], true, 16384, 40800.0),
        (
            "long32 cancellation",
            vec![40000, -40000],
            true,
            16384,
            800.0,
        ),
        ("normal small value", vec![2], false, 16384, 802.0),
        ("long small value", vec![2], true, 16384, 802.0),
    ];
    for (label, row, long, coordinate, ascent) in rows {
        let regions = vec![vec![ACTIVE]; row.len()];
        let data = mvar(&[*b"hasc"], &regions, &[row], long);
        run_font(
            label,
            with_mvar(&typo(&variable_ahem()), data),
            &[Query {
                size: 1000.0,
                coords: &[coordinate],
                complete_coords: &[coordinate],
                expected: [ascent, -200.0, 0.0, 234.0, 50.0],
            }],
            &mut failures,
        );
    }
    finish(failures);
}

#[test]
fn adjusted_strike_fallback_and_resolved_metric_signs_use_the_requested_instance() {
    let mut failures = Vec::new();
    let data = mvar(
        &[*b"hasc", *b"strs"],
        &[vec![ACTIVE]],
        &[vec![100], vec![-60]],
        false,
    );
    run_font(
        "varied nonpositive strike uses varied ascent",
        with_mvar(&typo(&variable_ahem()), data),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [18.0, -4.0, 0.0, 9.0, 20.0 / 18.0],
        }],
        &mut failures,
    );
    let no_os2 = remove_tables(
        &line_fields(&variable_ahem(), [800, -200, 7], [800, -200, 0], [800, 200]),
        &[*b"OS/2"],
    );
    run_font(
        "absent OS2 ignores tags and cannot acquire strike",
        with_mvar(&no_os2, primary()),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [16.0, -4.0, 0.14, 8.0, 20.0 / 18.0],
        }],
        &mut failures,
    );
    let repaired = typo(&line_fields(
        &variable_ahem(),
        [800, -200, 0],
        [-100, -200, 0],
        [800, 200],
    ));
    let data = mvar(&[*b"hasc"], &[vec![ACTIVE]], &[vec![200]], false);
    run_font(
        "resolved positive ascent admits negative base",
        with_mvar(&repaired, data),
        &[Query {
            size: 1000.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [100.0, -200.0, 0.0, 234.0, 50.0],
        }],
        &mut failures,
    );
    for size in [f32::from_bits(1), f32::MAX] {
        run_font(
            "f64 scaling keeps actual f32 size",
            variable_ahem(),
            &[Query {
                size,
                coords: &[8192],
                complete_coords: &[8192],
                expected: [
                    800.0 * (f64::from(size) / 1000.0),
                    -200.0 * (f64::from(size) / 1000.0),
                    0.0,
                    234.0 * (f64::from(size) / 1000.0),
                    50.0 * (f64::from(size) / 1000.0),
                ],
            }],
            &mut failures,
        );
    }
    finish(failures);
}

#[derive(Clone, Copy)]
enum Refusal {
    Table,
    Numeric,
}
fn reject_font(
    label: &str,
    bytes: Vec<u8>,
    coords: &[i16],
    kind: Refusal,
    failures: &mut Vec<String>,
) {
    let mut actual = system();
    let before = actual.font_catalogue_identity().clone();
    let stats = actual.stats();
    if let Err(error) = actual.register_fonts(bytes.clone()) {
        // This is an earlier public capability refusal, never executed metrics.
        eprintln!("{label} coords={coords:?}: registration before metrics: {error:?}");
        assert_eq!(error.code, ErrorCode::FontLoadFailed);
        assert_eq!(actual.font_catalogue_identity(), &before);
        assert_eq!(actual.stats(), stats);
        return;
    }
    let data = actual.registered_font_faces().next().unwrap().to_owned();
    let basis = actual.font_catalogue_identity().clone();
    let stats = actual.stats();
    let outcome = data
        .unsynthesized_instance(1000.0, coords)
        .unwrap()
        .metrics(&basis);
    assert_eq!(actual.stats(), stats);
    assert_eq!(actual.font_catalogue_identity(), &basis);
    assert_eq!(data.bytes(), bytes);
    match outcome {
        Ok(_) => failures.push(format!(
            "{label} {coords:?}: malformed or invalid resolved metrics admitted"
        )),
        Err(error) => {
            eprintln!("{label} coords={coords:?}: metric refusal: {error:?}");
            let correct_detail = match kind {
                Refusal::Table => matches!(
                    error.detail(),
                    Some(ErrorDetail::FontMetricTable {
                        table: FontMetricTable::Mvar,
                        ..
                    })
                ),
                Refusal::Numeric => {
                    matches!(error.detail(), Some(ErrorDetail::InvalidFontMetric { .. }))
                }
            };
            if error.code != ErrorCode::InvalidFontMetrics || !correct_detail {
                failures.push(format!(
                    "{label} {coords:?}: wrong metric refusal {error:?}"
                ));
            }
        }
    }
}

#[test]
fn malformed_present_rows_refuse_even_at_default_and_inactive_coordinates() {
    let good = primary();
    let mut cases = Vec::new();
    for (label, at, value) in [
        ("unsupported table version", 0, 2),
        ("reserved header", 4, 1),
        ("short record stride", 6, 7),
        ("extended record stride", 6, 9),
        ("declared record tail absent", 8, 6),
        ("nonnull records require store", 10, 0),
        ("wrong region axis inventory", 64, 2),
        ("region index out of bounds", 80, 1),
        ("word count exceeds region count", 76, 2),
        ("outer beyond declared slot array", 16, 1),
        ("nonnull inner FFFF out of bounds", 18, u16::MAX),
        ("half sentinel outer out of bounds", 16, u16::MAX),
        ("noncanonical tuple value", 70, 32767),
    ] {
        let mut data = good.clone();
        put_u16(&mut data, at, value);
        cases.push((label, data));
    }
    let mut data = good.clone();
    data[20..24].copy_from_slice(b"hasc");
    cases.push(("duplicate value tags", data));
    let mut data = good.clone();
    data[12..16].copy_from_slice(b"zzzz");
    cases.push(("unsorted value tags", data));
    let mut data = good.clone();
    put32(&mut data, 60, u32::MAX);
    cases.push(("inaccessible nonnull data offset", data));
    let mut data = good.clone();
    data.truncate(73);
    cases.push(("truncated region tuple", data));
    let mut data = good.clone();
    data.truncate(81);
    cases.push(("truncated region index", data));
    let mut data = good.clone();
    data.pop();
    cases.push(("last row delta truncated", data));
    // Unknown/unreturned tags still require a complete referenced row.
    let mut data = good.clone();
    data[44..48].copy_from_slice(b"zzzz");
    data.pop();
    cases.push(("unreturned tag has truncated row", data));
    let mut failures = Vec::new();
    for (label, data) in cases {
        for coords in [&[][..], &[-16384][..]] {
            reject_font(
                label,
                with_mvar(&typo(&variable_ahem()), data.clone()),
                coords,
                Refusal::Table,
                &mut failures,
            );
        }
    }
    reject_font(
        "MVAR without actual fvar",
        with_mvar(AHEM, good),
        &[],
        Refusal::Table,
        &mut failures,
    );
    finish(failures);
}

#[test]
fn invalid_resolved_signs_refuse_but_inapplicable_tags_do_not_execute_numeric_checks() {
    let mut failures = Vec::new();
    for (label, tag, delta) in [
        ("negative resolved ascent", *b"hasc", -901),
        ("positive resolved descent", *b"hdsc", 201),
    ] {
        let data = mvar(&[tag], &[vec![ACTIVE]], &[vec![delta]], false);
        reject_font(
            label,
            with_mvar(&typo(&variable_ahem()), data),
            &[16384],
            Refusal::Numeric,
            &mut failures,
        );
    }
    let windows = line_fields(&variable_ahem(), [0, 0, 7], [0, 0, 99], [900, 300]);
    let data = mvar(&[*b"hcld"], &[vec![ACTIVE]], &[vec![-301]], false);
    reject_font(
        "negative stored Windows descent is not sign reversed",
        with_mvar(&windows, data),
        &[16384],
        Refusal::Numeric,
        &mut failures,
    );
    let data = mvar(&[*b"hasc"], &[vec![ACTIVE]], &[vec![-1000]], false);
    run_font(
        "inapplicable typo delta does not invalidate hhea",
        with_mvar(&variable_ahem(), data),
        &[Query {
            size: 20.0,
            coords: &[16384],
            complete_coords: &[16384],
            expected: [16.0, -4.0, 0.0, 4.68, 1.0],
        }],
        &mut failures,
    );
    finish(failures);
}

#[test]
fn static_metrics_and_instance_input_validation_remain_executable_controls() {
    assert_eq!(
        u16_at(table(AHEM, *b"OS/2"), 0),
        3,
        "original licensed fixture is unchanged"
    );
    let mut failures = Vec::new();
    run_font(
        "static Ahem",
        AHEM.to_vec(),
        &[Query {
            size: 20.0,
            coords: &[],
            complete_coords: &[],
            expected: [16.0, -4.0, 0.0, 4.68, 1.0],
        }],
        &mut failures,
    );
    // Reuse the existing collection encoder and prove actual collection index,
    // rather than issuing any test-only raw FontData or catalogue token.
    let bytes = two_face_collection(AHEM.to_vec());
    let mut actual = system();
    actual.register_fonts(bytes.clone()).unwrap();
    let faces: Vec<_> = actual
        .registered_font_faces()
        .map(|face| face.to_owned())
        .collect();
    assert_eq!(
        faces.iter().map(FontData::index).collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(!faces[0].face().is_same_face(&faces[1].face()));
    for data in faces {
        query(
            "static collection face",
            &actual,
            &data,
            Query {
                size: 20.0,
                coords: &[],
                complete_coords: &[],
                expected: [16.0, -4.0, 0.0, 4.68, 1.0],
            },
            &mut failures,
        );
        assert_eq!(data.bytes(), bytes);
    }
    let mut actual = system();
    actual.register_fonts(variable_ahem()).unwrap();
    let data = actual.registered_font_faces().next().unwrap().to_owned();
    let stats = actual.stats();
    for coords in [&[0, 0][..], &[16385][..], &[-16385][..]] {
        assert_eq!(
            data.unsynthesized_instance(20.0, coords).unwrap_err().code,
            ErrorCode::InvalidFontInstance
        );
    }
    for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(
            data.unsynthesized_instance(size, &[]).unwrap_err().code,
            ErrorCode::InvalidFontInstance
        );
    }
    assert_eq!(actual.stats(), stats);
    finish(failures);
}

#[test]
fn catalogue_mismatch_precedes_variation_and_retained_resource_uses_original_basis() {
    let mut actual = system();
    let bytes = with_mvar(&typo(&variable_ahem()), primary());
    actual.register_fonts(bytes.clone()).unwrap();
    let data = actual.registered_font_faces().next().unwrap().to_owned();
    let old = data.catalogue_identity().clone();
    actual.refresh_fonts().unwrap();
    let fresh = actual.registered_font_faces().next().unwrap().to_owned();
    assert!(data.face().is_same_face(&fresh.face()));
    assert_ne!(data.catalogue_identity(), fresh.catalogue_identity());
    let stats = actual.stats();
    let error = data
        .unsynthesized_instance(20.0, &[8192])
        .unwrap()
        .metrics(actual.font_catalogue_identity())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FontCatalogueMismatch);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontCatalogueMismatch {
            expected: actual.font_catalogue_identity().clone(),
            actual: old.clone()
        })
    );
    let foreign = system();
    assert_eq!(
        data.unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(foreign.font_catalogue_identity())
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    // A malformed present MVAR must still lose to a mismatched publication.
    let mut other = system();
    other
        .register_fonts(with_mvar(&variable_ahem(), vec![0]))
        .unwrap();
    let malformed = other.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(
        malformed
            .unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(&old)
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    assert_eq!(actual.stats(), stats);
    drop(actual);
    let coords = [8192];
    let result = data
        .unsynthesized_instance(20.0, &coords)
        .unwrap()
        .metrics(&old);
    assert_eq!(data.bytes(), bytes);
    let mut failures = Vec::new();
    match result {
        Err(error) => failures.push(format!(
            "retained original basis after System drop: {error:?}"
        )),
        Ok(metrics) => {
            assert!(metrics.instance().face().is_same_face(&data.face()));
            assert_eq!(
                metrics.instance().normalized_coords().collect::<Vec<_>>(),
                [8192]
            );
            assert_eq!(
                metrics.instance().face().bytes().as_ptr(),
                data.bytes().as_ptr()
            );
            close("retained ascent", metrics.ascent(), 17.0);
            close("retained strike", metrics.strikeout().center_offset(), 5.03);
        }
    }
    finish(failures);
}
