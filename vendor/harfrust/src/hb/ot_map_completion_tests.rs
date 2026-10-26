use super::*;
use crate::hb::completion_fixture::{gsub_lookup, with_font, with_gsub, with_tables, words};
use crate::{Feature, ShapingRefusal, ShapingStatus, UnicodeBuffer};

fn alternate_font(f: impl FnOnce(&crate::Shaper<'_>, u32)) {
    let mut original = 0;
    with_font(|font| original = font.get_nominal_glyph('A' as u32).unwrap().to_u32());
    let mut table = words(&[1, 522, 1, 8, 256]);
    table.extend(words(&[2; 256]));
    table.extend(words(&[1, 1, original as u16]));
    let gsub = gsub_lookup(3, &table);
    with_gsub(&gsub, |font| f(font, original));
}

#[test]
fn real_selected_feature_value_capacity_is_reported_on_actual_result() {
    alternate_font(|font, original| {
        for value in [255, 256, 257] {
            let feature = Feature::new(hb_tag_t::new(b"ccmp"), value, ..);
            let mut input = UnicodeBuffer::new();
            input.push_str("A");
            input.guess_segment_properties();
            let result = font.shape(input, &[feature]);
            assert_eq!(result.len(), 1);
            // Preserve original mask semantics, including partial high-bit loss.
            assert_eq!(
                result.glyph_infos()[0].glyph_id,
                if value == 256 { original } else { 2 }
            );
            if value == 255 {
                assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            } else {
                assert_ne!(result.shaping_status(), ShapingStatus::Complete);
                assert_eq!(
                    result.shaping_status(),
                    ShapingStatus::Refused(ShapingRefusal::FeatureCapacityLimit)
                );
            }
        }
    });
}

fn input(text: &str) -> UnicodeBuffer {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    buffer
}

#[test]
fn plan_qualification_is_conservative_but_excludes_disabled_absent_and_empty() {
    alternate_font(|font, _| {
        let global = |value| Feature::new(hb_tag_t::new(b"ccmp"), value, ..);
        let range = |value, start, end| Feature {
            tag: hb_tag_t::new(b"ccmp"),
            value,
            start,
            end,
        };
        for features in [
            alloc::vec![range(256, 0, 1), global(1)],
            alloc::vec![range(256, 10, 11)],
            alloc::vec![global(256), range(0, 0, 1)],
            alloc::vec![global(256), range(1, 0, 1)],
        ] {
            assert_eq!(
                font.shape(input("A"), &features).shaping_status(),
                ShapingStatus::Refused(ShapingRefusal::FeatureCapacityLimit)
            );
        }
        for features in [
            alloc::vec![global(256), global(0)],
            alloc::vec![Feature::new(hb_tag_t::new(b"zzzz"), 256, ..)],
        ] {
            assert_eq!(
                font.shape(input("A"), &features).shaping_status(),
                ShapingStatus::Complete
            );
        }
        assert_eq!(
            font.shape(input(""), &[global(256)]).shaping_status(),
            ShapingStatus::Complete
        );
    });
    with_font(|font| {
        // Neither a stub fallback nor the dummy trak mask proves real feature work.
        for tag in [b"kern", b"trak"] {
            assert_eq!(
                font.shape(input("AA"), &[Feature::new(hb_tag_t::new(tag), 256, ..)])
                    .shaping_status(),
                ShapingStatus::Complete
            );
        }
    });
}

fn feature_font(tags: &[[u8; 4]], gid: u16) -> Vec<u8> {
    let count = u16::try_from(tags.len()).unwrap();
    let feature_offset = 28 + 2 * count;
    let lookup_offset = feature_offset + 2 + 12 * count;
    let mut data = words(&[1, 0, 10, feature_offset, lookup_offset, 1]);
    data.extend_from_slice(b"DFLT");
    data.extend(words(&[8, 4, 0, 0, 0xFFFF, count]));
    for index in 0..count {
        data.extend(words(&[index]));
    }
    data.extend(words(&[count]));
    for (index, tag) in tags.iter().enumerate() {
        data.extend_from_slice(tag);
        data.extend(words(&[2 + 6 * count + 6 * index as u16]));
    }
    for _ in tags {
        data.extend(words(&[0, 1, 0]));
    }
    data.extend(words(&[1, 4, 1, 0, 1, 8, 1, 6, 0, 1, 1, gid]));
    data
}

#[test]
fn total_mask_capacity_and_builtin_default_zero_loss_are_observable() {
    let mut gid = 0;
    with_font(|font| gid = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16);
    let tags = [*b"a000", *b"a001", *b"a002", *b"a003"];
    for count in [3, 4] {
        let gsub = feature_font(&tags[..count], gid);
        with_gsub(&gsub, |font| {
            let features: Vec<_> = tags[..count]
                .iter()
                .map(|tag| Feature {
                    tag: hb_tag_t::new(tag),
                    value: 255,
                    start: 0,
                    end: 1,
                })
                .collect();
            assert_eq!(
                font.shape(input("A"), &features).shaping_status(),
                if count == 3 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::FeatureCapacityLimit)
                }
            );
        });
    }
    let tags = [*b"a000", *b"a001", *b"a002", *b"b000", *b"b001", *b"frac"];
    let gsub = feature_font(&tags, gid);
    with_gsub(&gsub, |font| {
        let features: Vec<_> = tags[..5]
            .iter()
            .enumerate()
            .map(|(index, tag)| Feature {
                tag: hb_tag_t::new(tag),
                value: if index < 3 { 255 } else { 1 },
                start: 0,
                end: 1,
            })
            .collect();
        let plan =
            crate::ShapePlan::new(font, crate::Direction::LeftToRight, None, None, &features);
        assert_eq!(plan.ot_map.get_mask(hb_tag_t::new(b"frac")).0, 0);
        assert_eq!(
            font.shape(input("1/2"), &features).shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::FeatureCapacityLimit)
        );
    });
}

#[test]
fn unused_gsub_capacity_loss_does_not_refuse_selected_morx() {
    let mut gid = 0;
    with_font(|font| gid = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16);
    let gsub = gsub_lookup(1, &words(&[1, 6, 0, 1, 1, gid]));
    let morx: Vec<u8> = [0x0002_0000u32, 0]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    with_tables(&[(*b"GSUB", &gsub), (*b"morx", &morx)], |font| {
        let features = [Feature::new(hb_tag_t::new(b"ccmp"), 256, ..)];
        let mut source = input("A");
        source.set_script(crate::script::UNKNOWN);
        let plan =
            crate::ShapePlan::new(font, source.0.direction, source.0.script, None, &features);
        assert!(plan.apply_morx);
        assert_eq!(
            font.shape_with_plan(&plan, source, &features)
                .shaping_status(),
            ShapingStatus::Complete
        );
    });
}

#[test]
fn actual_native_kern_and_kerx_mask_consumers_qualify_capacity_loss() {
    let mut gid = 0;
    with_font(|font| gid = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16);
    let kern = words(&[0, 1, 0, 20, 1, 1, 6, 0, 0, gid, gid, 50]);
    let mut kerx: Vec<u8> = [0x0002_0000u32, 1, 34, 0, 0, 1, 6, 0, 0]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    kerx.extend(words(&[gid, gid, 50]));
    for (tag, table) in [(*b"kern", kern.as_slice()), (*b"kerx", kerx.as_slice())] {
        with_tables(&[(tag, table)], |font| {
            let disabled = font.shape(input("AA"), &[Feature::new(hb_tag_t::new(b"kern"), 0, ..)]);
            let base: i32 = disabled.glyph_positions().iter().map(|p| p.x_advance).sum();
            for value in [255, 256, 257] {
                let features = [Feature::new(hb_tag_t::new(b"kern"), value, ..)];
                let plan = crate::ShapePlan::new(
                    font,
                    crate::Direction::LeftToRight,
                    Some(crate::script::LATIN),
                    None,
                    &features,
                );
                assert!(plan.apply_kern || plan.apply_kerx);
                let result = font.shape(input("AA"), &features);
                let advance: i32 = result.glyph_positions().iter().map(|p| p.x_advance).sum();
                assert_eq!(advance, base + if value == 256 { 0 } else { 50 });
                assert_eq!(
                    result.shaping_status(),
                    if value == 255 {
                        ShapingStatus::Complete
                    } else {
                        ShapingStatus::Refused(ShapingRefusal::FeatureCapacityLimit)
                    }
                );
            }
        });
    }
}

#[test]
fn mask_independent_native_crossstream_and_format4_do_not_refuse() {
    let mut gid = 0;
    with_font(|font| gid = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16);
    let mut machine = words(&[4, 8, 14, 18, gid, 1]);
    machine.extend_from_slice(&[1, 0, 0, 1, 1, 1]);
    machine.extend(words(&[14, 0, 14, 0x8000 | 0x001a, 3]));
    let mut kern = words(&[0, 1, 0, 34, 0x0105]);
    kern.extend(machine);
    let mut bodies = Vec::new();
    for kind in [1u32, 4] {
        // Actual mark/push entries make the glyph eligible for this machine.
        // Format1 crosses the stream; Format4 marks without a mask-based action.
        let end = if kind == 1 { 50u32 } else { 48 };
        let mut body: Vec<u8> = [4u32, 20, 28, 36, 48]
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect();
        body.extend(words(&[
            8,
            gid,
            1,
            1,
            0,
            1,
            1,
            1,
            0,
            0,
            0xFFFF,
            0,
            0x8000,
            if kind == 1 { 0 } else { 0xFFFF },
        ]));
        if kind == 1 {
            body.extend(words(&[3]));
        }
        assert_eq!(body.len(), end as usize);
        let mut kerx: Vec<u8> = [
            0x0002_0000u32,
            1,
            12 + end,
            if kind == 1 { 0x4000_0001 } else { 4 },
            0,
        ]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
        kerx.extend(body);
        bodies.push(kerx);
    }
    for (index, (tag, table)) in [
        (*b"kern", kern.as_slice()),
        (*b"kerx", bodies[0].as_slice()),
        (*b"kerx", bodies[1].as_slice()),
    ]
    .into_iter()
    .enumerate()
    {
        with_tables(&[(tag, table)], |font| {
            let features = [Feature::new(hb_tag_t::new(b"kern"), 256, ..)];
            let plan = crate::ShapePlan::new(
                font,
                crate::Direction::LeftToRight,
                Some(crate::script::LATIN),
                None,
                &features,
            );
            assert!(plan.apply_kern || plan.apply_kerx);
            let result = font.shape(input("AA"), &features);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            if index < 2 {
                assert!(result.glyph_positions().iter().any(|p| p.y_offset != 0));
            }
        });
    }
}

#[test]
fn real_gpos_kern_mask_loss_preserves_kerx_choice_and_reports_plan_refusal() {
    let mut gid = 0;
    with_font(|font| gid = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16);
    let tags = [*b"a000", *b"a001", *b"a002"];
    let gsub = feature_font(&tags, gid);
    let mut gpos = gsub_lookup(1, &words(&[1, 8, 4, 50, 1, 1, gid]));
    gpos[32..36].copy_from_slice(b"kern");
    let mut kerx: Vec<u8> = [0x0002_0000u32, 1, 34, 0, 0, 1, 6, 0, 0]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    kerx.extend(words(&[gid, gid, 25]));
    with_tables(
        &[(*b"GSUB", &gsub), (*b"GPOS", &gpos), (*b"kerx", &kerx)],
        |font| {
            for count in [2, 3] {
                let mut features: Vec<_> = tags[..count]
                    .iter()
                    .map(|tag| Feature {
                        tag: hb_tag_t::new(tag),
                        value: 255,
                        start: 0,
                        end: 1,
                    })
                    .collect();
                features.push(Feature::new(hb_tag_t::new(b"kern"), 255, ..));
                let source = input("AA");
                let plan = crate::ShapePlan::new(
                    font,
                    source.0.direction,
                    source.0.script,
                    None,
                    &features,
                );
                assert!(plan.apply_gpos);
                assert_eq!(
                    plan.ot_map
                        .get_feature_index(TableIndex::GPOS, hb_tag_t::new(b"kern"))
                        .is_some(),
                    count == 2
                );
                assert_eq!(plan.apply_kerx, count == 3);
                let result = font.shape_with_plan(&plan, source, &features);
                // Preserve GPOS-first dispatch even when the lost feature changed the
                // retained kerx-choice flag. Never substitute a different producer.
                let advance: i32 = result.glyph_positions().iter().map(|p| p.x_advance).sum();
                assert_eq!(advance, if count == 2 { 2100 } else { 2000 });
                assert_eq!(
                    result.shaping_status(),
                    if count == 2 {
                        ShapingStatus::Complete
                    } else {
                        ShapingStatus::Refused(ShapingRefusal::FeatureCapacityLimit)
                    }
                );
            }
        },
    );
}
