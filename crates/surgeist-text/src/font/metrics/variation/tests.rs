use super::*;
use crate::{
    FontCatalogueMode, FontData as Resource, System, SystemOptions,
    test_fonts::{replace_tables, table, variable_ahem},
};

// Binary encoding only. One actual fvar axis; every region is [0,1,1].
fn one_record(deltas: &[i32], long: bool) -> Vec<u8> {
    let regions = u16::try_from(deltas.len()).unwrap();
    let mut bytes = Vec::new();
    for value in [1u16, 0, 0, 8, 1, 20] {
        bytes.extend(value.to_be_bytes());
    }
    bytes.extend(*b"hasc");
    bytes.extend([0u8; 4]);
    bytes.extend(1u16.to_be_bytes());
    bytes.extend(12u32.to_be_bytes());
    bytes.extend(1u16.to_be_bytes());
    bytes.extend((16u32 + u32::from(regions) * 6).to_be_bytes());
    for value in [1, regions] {
        bytes.extend(value.to_be_bytes());
    }
    for _ in deltas {
        for value in [0i16, 16384, 16384] {
            bytes.extend(value.to_be_bytes());
        }
    }
    for value in [1, regions | if long { 0x8000 } else { 0 }, regions] {
        bytes.extend(value.to_be_bytes());
    }
    for index in 0..regions {
        bytes.extend(index.to_be_bytes());
    }
    for &value in deltas {
        if long {
            bytes.extend(value.to_be_bytes());
        } else {
            bytes.extend(i16::try_from(value).unwrap().to_be_bytes());
        }
    }
    bytes
}
fn fixture(data: Vec<u8>) -> Vec<u8> {
    let base = variable_ahem();
    let mut os2 = table(&base, *b"OS/2").to_vec();
    os2[..2].copy_from_slice(&4u16.to_be_bytes());
    let flags = u16::from_be_bytes(os2[62..64].try_into().unwrap()) | 0x80;
    os2[62..64].copy_from_slice(&flags.to_be_bytes());
    replace_tables(&base, vec![(*b"OS/2", os2), (*b"MVAR", data)])
}
fn registered(bytes: Vec<u8>) -> (System, Resource) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let resource = system.registered_font_faces().next().unwrap().to_owned();
    (system, resource)
}

#[test]
fn actual_shared_offsets_and_rows_are_valid_across_distinct_outer_indices() {
    // Two sorted tags, two declared outer slots aliasing one real data row.
    let mut data = Vec::new();
    for value in [1u16, 0, 0, 8, 2, 28] {
        data.extend(value.to_be_bytes());
    }
    for (tag, outer) in [(*b"hasc", 0u16), (*b"hdsc", 1u16)] {
        data.extend(tag);
        data.extend(outer.to_be_bytes());
        data.extend(0u16.to_be_bytes());
    }
    data.extend(1u16.to_be_bytes());
    data.extend(16u32.to_be_bytes());
    data.extend(2u16.to_be_bytes());
    data.extend(26u32.to_be_bytes());
    data.extend(26u32.to_be_bytes());
    for value in [1u16, 1, 0, 16384, 16384, 1, 1, 1, 0, 100] {
        data.extend(value.to_be_bytes());
    }
    let (system, resource) = registered(fixture(data));
    let stats = system.stats();
    let metrics = resource
        .unsynthesized_instance(1000.0, &[16384])
        .unwrap()
        .metrics(system.font_catalogue_identity())
        .unwrap();
    assert_eq!(
        (metrics.ascent(), metrics.descent(), metrics.leading()),
        (900.0, -100.0, 0.0)
    );
    assert_eq!(system.stats(), stats);
}

#[test]
fn actual_signed32_extremes_accumulate_without_narrowing_or_region_rounding() {
    for (values, coords, expected) in [
        ([i32::MAX, i32::MAX], 16384, 4_294_968_094.0),
        ([i32::MIN, i32::MAX], 16384, 799.0),
        ([i32::MIN, i32::MAX], 8192, 800.0),
    ] {
        let (system, resource) = registered(fixture(one_record(&values, true)));
        let requested = [coords];
        let metrics = resource
            .unsynthesized_instance(1000.0, &requested)
            .unwrap()
            .metrics(system.font_catalogue_identity())
            .unwrap();
        assert_eq!(metrics.ascent(), expected);
        assert_eq!(metrics.descent(), -200.0);
    }
}

#[test]
fn exact_mvar_refusals_and_capacity_owner_do_not_mutate_actual_inputs() {
    let good = one_record(&[100], false);
    let (system, resource) = registered(fixture(good.clone()));
    let stats = system.stats();
    let basis = system.font_catalogue_identity().clone();
    let instance = resource.unsynthesized_instance(20.0, &[]).unwrap();
    let mut cases = Vec::new();
    let mut data = good.clone();
    data[6..8].copy_from_slice(&9u16.to_be_bytes());
    cases.push((data, TableError::UnsupportedRecordSize { size: 9 }));
    let mut data = good.clone();
    data[16..18].copy_from_slice(&1u16.to_be_bytes());
    cases.push((
        data,
        TableError::VariationDeltaIndex {
            record: 0,
            outer: 1,
            inner: 0,
        },
    ));
    let mut data = good.clone();
    data[18..20].copy_from_slice(&u16::MAX.to_be_bytes());
    cases.push((
        data,
        TableError::VariationDeltaIndex {
            record: 0,
            outer: 0,
            inner: u16::MAX,
        },
    ));
    let mut data = good.clone();
    data[44..46].copy_from_slice(&2u16.to_be_bytes());
    cases.push((
        data,
        TableError::VariationWordCount {
            offset: 22,
            words: 2,
            regions: 1,
        },
    ));
    for (data, expected) in cases {
        let retained = data.clone();
        let error = resolve(Some(FontData::new(&data)), &instance, &[*b"hasc"])
            .err()
            .unwrap();
        assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::FontMetricTable {
                table: Table::Mvar,
                reason: expected
            })
        );
        assert_eq!(data, retained);
    }
    assert_eq!(system.stats(), stats);
    assert_eq!(system.font_catalogue_identity(), &basis);
    let mut coords = vec![NormalizedCoord::from_bits(8192)];
    let error = reserve(&mut coords, usize::MAX).unwrap_err();
    assert_eq!(error.code, ErrorCode::FontMetricCapacity);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontMetricCapacityOverflow)
    );
    assert_eq!(coords, [NormalizedCoord::from_bits(8192)]);
}
