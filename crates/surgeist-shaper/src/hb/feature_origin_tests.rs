//! New feature-intent API qualification using authored real OpenType tables.
//! No executable preimplementation boundary expressed optional-default intent.
use super::*;
use crate::hb::common::FeatureOrigin;
use crate::hb::completion_fixture::{gsub_lookup, with_font, with_gsub, words};
use crate::{Direction, ShapePlan, ShapePlanKey, ShapingStatus, UnicodeBuffer};

fn input(text: &str, script: Script) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    for (index, ch) in text.chars().enumerate() {
        input.add(ch, u32::try_from(index).unwrap());
    }
    input.set_direction(Direction::LeftToRight);
    input.set_script(script);
    input.observe_components();
    input
}

fn ligature_table(tag: &[u8; 4], first: u16, second: u16, replacement: u16) -> Vec<u8> {
    let mut gsub = gsub_lookup(
        4,
        &words(&[1, 18, 1, 8, 1, 4, replacement, 2, second, 1, 1, first]),
    );
    gsub[32..36].copy_from_slice(tag);
    gsub
}

fn with_ligature(tag: &[u8; 4], test: impl FnOnce(&crate::Shaper<'_>, u16)) {
    let mut a = 0;
    let mut b = 0;
    with_font(|font| {
        a = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16;
        b = font.get_nominal_glyph('B' as u32).unwrap().to_u32() as u16;
    });
    with_gsub(&ligature_table(tag, a, a, b), |font| test(font, b));
}

#[test]
fn constructors_and_parser_preserve_exact_bounds_and_distinct_intent() {
    let tag = hb_tag_t::new(b"liga");
    let explicit = Feature::new(tag, 0, 3..=5);
    let soft = Feature::optional_default(tag, 0, 3..=5);
    assert_eq!(explicit.origin(), FeatureOrigin::Explicit);
    assert_eq!(soft.origin(), FeatureOrigin::OptionalDefault);
    assert_ne!(explicit, soft);
    assert_eq!((explicit.start, explicit.end), (3, 6));
    assert_eq!((soft.start, soft.end), (3, 6));
    for text in ["liga[3:5]=0", "liga[-1]", "liga[5:3]", "liga[0:0]"] {
        assert_eq!(
            text.parse::<Feature>().unwrap().origin(),
            FeatureOrigin::Explicit
        );
    }
    assert!("liga[".parse::<Feature>().is_err());
}

#[test]
fn selected_optional_liga_and_clig_defaults_suppress_only_touched_endpoints() {
    for tag in [b"liga", b"clig"] {
        with_ligature(tag, |font, replacement| {
            let ordinary = font.shape(input("AAAA", crate::script::LATIN), &[]);
            assert_eq!(
                ordinary
                    .glyph_infos()
                    .iter()
                    .map(|g| g.glyph_id)
                    .collect::<Vec<_>>(),
                [u32::from(replacement); 2]
            );
            let feature = Feature::optional_default(hb_tag_t::new(tag), 0, 0..2);
            let result = font.shape(input("AAAA", crate::script::LATIN), &[feature]);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(result.len(), 3);
            assert_eq!(result.glyph_infos()[2].glyph_id, u32::from(replacement));
            let receipt = result.component_receipt().unwrap().unwrap();
            assert_eq!(receipt.default_features()[0].request(), feature);
            assert_eq!(
                receipt.default_features()[0].disposition(),
                DefaultFeatureDisposition::Accepted
            );
            let relation = receipt.ligatures().next().unwrap();
            assert_eq!(relation.components(), [2, 3]);
            assert!(relation
                .provenance()
                .iter()
                .any(|p| p.tag() == hb_tag_t::new(tag)
                    && p.role() == FeatureRole::Optional
                    && p.source() == FeatureSource::Provider));
        });
    }
}

#[test]
fn explicit_global_and_ranged_values_override_soft_defaults_in_any_input_order() {
    with_ligature(b"liga", |font, replacement| {
        let tag = hb_tag_t::new(b"liga");
        let soft = Feature::optional_default(tag, 0, 0..2);
        let global = Feature::new(tag, 1, ..);
        let ranged = Feature::new(tag, 1, 0..2);
        for features in [
            [soft, global],
            [global, soft],
            [soft, ranged],
            [ranged, soft],
        ] {
            let result = font.shape(input("AA", crate::script::LATIN), &features);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(result.len(), 1);
            assert_eq!(result.glyph_infos()[0].glyph_id, u32::from(replacement));
            let receipt = result.component_receipt().unwrap().unwrap();
            let provenance = receipt.ligatures().next().unwrap().provenance().to_vec();
            assert!(provenance
                .iter()
                .any(|p| matches!(p.source(), FeatureSource::Request(request)
                if request.origin() == FeatureOrigin::Explicit && request.value == 1)));
            assert!(!provenance
                .iter()
                .any(|p| matches!(p.source(), FeatureSource::Request(request)
                if request.origin() == FeatureOrigin::OptionalDefault)));
        }
    });
}

#[test]
fn last_same_origin_overlap_wins_for_global_and_range_replay() {
    with_ligature(b"liga", |font, _| {
        let tag = hb_tag_t::new(b"liga");
        for features in [
            [
                Feature::optional_default(tag, 0, 0..2),
                Feature::optional_default(tag, 1, ..),
            ],
            [
                Feature::optional_default(tag, 0, ..),
                Feature::optional_default(tag, 1, 0..2),
            ],
            [Feature::new(tag, 0, 0..2), Feature::new(tag, 1, ..)],
            [Feature::new(tag, 0, ..), Feature::new(tag, 1, 0..2)],
        ] {
            assert_eq!(
                font.shape(input("AA", crate::script::LATIN), &features)
                    .len(),
                1
            );
            let reversed = [features[1], features[0]];
            assert_eq!(
                font.shape(input("AA", crate::script::LATIN), &reversed)
                    .len(),
                2
            );
        }
    });
}

#[test]
fn required_and_unknown_soft_requests_do_not_allocate_or_disable_features() {
    with_ligature(b"rlig", |font, _| {
        let request = Feature::optional_default(hb_tag_t::new(b"rlig"), 256, 0..2);
        let result = font.shape(input("AA", crate::script::LATIN), &[request]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(result.len(), 1);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(
            receipt.default_features()[0].disposition(),
            DefaultFeatureDisposition::Required
        );
        assert!(receipt
            .ligatures()
            .next()
            .unwrap()
            .provenance()
            .iter()
            .any(|p| p.tag() == hb_tag_t::new(b"rlig") && p.role() == FeatureRole::Required));
    });
    with_ligature(b"zzzz", |font, _| {
        let explicit = Feature::new(hb_tag_t::new(b"zzzz"), 1, ..);
        let soft = Feature::optional_default(hb_tag_t::new(b"zzzz"), 256, 0..2);
        let result = font.shape(input("AA", crate::script::LATIN), &[explicit, soft]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(result.len(), 1);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(
            receipt.default_features()[0].disposition(),
            DefaultFeatureDisposition::Unclassified
        );
        assert!(receipt
            .ligatures()
            .next()
            .unwrap()
            .provenance()
            .iter()
            .any(|p| p.role() == FeatureRole::Unclassified
                && p.source() == FeatureSource::Request(explicit)));
    });
}

#[test]
fn actual_required_langsys_overrides_optional_role_and_retains_its_source() {
    let mut a = 0;
    let mut b = 0;
    with_font(|font| {
        a = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16;
        b = font.get_nominal_glyph('B' as u32).unwrap().to_u32() as u16;
    });
    let mut gsub = ligature_table(b"liga", a, a, b);
    gsub[24..26].copy_from_slice(&0u16.to_be_bytes());
    with_gsub(&gsub, |font| {
        let request = Feature::optional_default(hb_tag_t::new(b"liga"), 0, ..);
        let result = font.shape(input("AA", crate::script::LATIN), &[request]);
        assert_eq!(result.len(), 1);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(
            receipt.default_features()[0].disposition(),
            DefaultFeatureDisposition::Required
        );
        assert!(receipt
            .ligatures()
            .next()
            .unwrap()
            .provenance()
            .iter()
            .any(|p| p.role() == FeatureRole::Required
                && p.source() == FeatureSource::LanguageSystem));
    });
}

#[test]
fn khmer_required_clig_override_keeps_actual_ligature_and_script_provenance() {
    // Ahem maps these actual Khmer consonants to glyph0. The authored GSUB
    // supplies the real clig route under the actual Khmer shaper, independently
    // of the font's coverage/artwork for those characters.
    with_gsub(&ligature_table(b"clig", 0, 0, 1), |font| {
        let soft = Feature::optional_default(hb_tag_t::new(b"clig"), 0, 0..2);
        for explicit in [
            Feature::new(hb_tag_t::new(b"clig"), 0, ..),
            Feature::new(hb_tag_t::new(b"clig"), 0, 0..2),
        ] {
            let result = font.shape(
                input("\u{1780}\u{1781}", crate::script::KHMER),
                &[soft, explicit],
            );
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(result.len(), 1);
            let receipt = result.component_receipt().unwrap().unwrap();
            assert_eq!(
                receipt.default_features()[0].disposition(),
                DefaultFeatureDisposition::Required
            );
            assert!(receipt
                .ligatures()
                .next()
                .unwrap()
                .provenance()
                .iter()
                .any(|p| p.tag() == hb_tag_t::new(b"clig")
                    && p.role() == FeatureRole::Required
                    && p.source() == FeatureSource::Script));
        }
    });
}

#[test]
fn shape_plan_key_compares_exact_bounds_origin_and_canonical_origin_order() {
    with_ligature(b"liga", |font, _| {
        let tag = hb_tag_t::new(b"liga");
        let soft = Feature::optional_default(tag, 0, 0..2);
        let explicit = Feature::new(tag, 1, ..);
        let plan = ShapePlan::new(
            font,
            Direction::LeftToRight,
            Some(crate::script::LATIN),
            None,
            &[soft, explicit],
        );
        let key = || ShapePlanKey::new(Some(crate::script::LATIN), Direction::LeftToRight);
        assert!(key().features(&[explicit, soft]).matches(&plan));
        assert!(!key()
            .features(&[Feature::optional_default(tag, 0, 1..2), explicit])
            .matches(&plan));
        assert!(!key()
            .features(&[Feature::new(tag, 0, 0..2), explicit])
            .matches(&plan));
        assert!(!key()
            .features(&[soft, Feature::new(tag, 1, 0..2)])
            .matches(&plan));
    });
}

fn shared_lookup_table(tags: &[[u8; 4]], a: u16, replacement: u16) -> Vec<u8> {
    let count = u16::try_from(tags.len()).unwrap();
    let mut scripts = words(&[1]);
    scripts.extend_from_slice(b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xFFFF, count]));
    scripts.extend(words(&(0..count).collect::<Vec<_>>()));
    let mut features = words(&[count]);
    for (index, tag) in tags.iter().enumerate() {
        features.extend_from_slice(tag);
        features.extend(words(&[2 + 6 * count + 6 * index as u16]));
    }
    for _ in tags {
        features.extend(words(&[0, 1, 0]));
    }
    let mut data = words(&[
        1,
        0,
        10,
        10 + scripts.len() as u16,
        10 + scripts.len() as u16 + features.len() as u16,
    ]);
    data.extend(scripts);
    data.extend(features);
    data.extend(words(&[1, 4, 4, 0, 1, 8]));
    data.extend(words(&[1, 18, 1, 8, 1, 4, replacement, 2, a, 1, 1, a]));
    data
}

#[test]
fn merged_lookup_retains_every_enabled_source_and_excludes_disabled_contributions() {
    let mut a = 0;
    let mut b = 0;
    with_font(|font| {
        a = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16;
        b = font.get_nominal_glyph('B' as u32).unwrap().to_u32() as u16;
    });
    with_gsub(&shared_lookup_table(&[*b"clig", *b"liga"], a, b), |font| {
        for features in [
            Vec::new(),
            alloc::vec![Feature::new(hb_tag_t::new(b"liga"), 0, ..)],
        ] {
            let result = font.shape(input("AA", crate::script::LATIN), &features);
            assert_eq!(result.len(), 1);
            let receipt = result.component_receipt().unwrap().unwrap();
            let relation = receipt.ligatures().next().unwrap();
            assert!(relation
                .provenance()
                .iter()
                .any(|p| p.tag() == hb_tag_t::new(b"clig")));
            assert_eq!(
                relation
                    .provenance()
                    .iter()
                    .any(|p| p.tag() == hb_tag_t::new(b"liga")),
                features.is_empty()
            );
            assert!(relation
                .provenance()
                .iter()
                .all(|p| p.role() == FeatureRole::Optional));
        }
        // The same merged lookup is active under different features on each
        // actual component; the effective provenance is their union.
        let liga = Feature::new(hb_tag_t::new(b"liga"), 1, 0..1);
        let clig = Feature::new(hb_tag_t::new(b"clig"), 1, 1..2);
        let features = [
            Feature::new(hb_tag_t::new(b"liga"), 0, ..),
            Feature::new(hb_tag_t::new(b"clig"), 0, ..),
            liga,
            clig,
        ];
        let result = font.shape(input("AA", crate::script::LATIN), &features);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(result.len(), 1);
        let receipt = result.component_receipt().unwrap().unwrap();
        let relation = receipt.ligatures().next().unwrap();
        for request in [liga, clig] {
            assert!(relation
                .provenance()
                .iter()
                .any(|p| p.source() == FeatureSource::Request(request)));
        }
    });
}

#[test]
fn discretionary_and_historical_ligatures_remain_optional_and_default_disabled() {
    for tag in [b"dlig", b"hlig"] {
        with_ligature(tag, |font, _| {
            assert_eq!(font.shape(input("AA", crate::script::LATIN), &[]).len(), 2);
            let explicit = Feature::new(hb_tag_t::new(tag), 1, ..);
            let soft = Feature::optional_default(hb_tag_t::new(tag), 0, ..);
            let result = font.shape(input("AA", crate::script::LATIN), &[explicit, soft]);
            assert_eq!(result.len(), 1);
            let receipt = result.component_receipt().unwrap().unwrap();
            assert_eq!(
                receipt.default_features()[0].disposition(),
                DefaultFeatureDisposition::Accepted
            );
            assert!(receipt
                .ligatures()
                .next()
                .unwrap()
                .provenance()
                .iter()
                .all(|p| p.role() == FeatureRole::Optional
                    && p.source() == FeatureSource::Request(explicit)));
        });
    }
}

#[test]
fn reusable_and_supplied_vertical_front_doors_keep_exact_request_provenance() {
    with_ligature(b"liga", |font, replacement| {
        let tag = hb_tag_t::new(b"liga");
        let soft = Feature::optional_default(tag, 0, 0..2);
        let explicit = Feature::new(tag, 1, ..);
        let canonical = [soft, explicit];
        let reordered = [explicit, soft];
        let plan = ShapePlan::new(
            font,
            Direction::LeftToRight,
            Some(crate::script::LATIN),
            None,
            &canonical,
        );
        let result = font.shape_with_plan(&plan, input("AA", crate::script::LATIN), &reordered);
        assert_eq!(result.len(), 1);
        assert_eq!(result.glyph_infos()[0].glyph_id, u32::from(replacement));
        let receipt = result.component_receipt().unwrap().unwrap();
        assert!(receipt
            .ligatures()
            .next()
            .unwrap()
            .provenance()
            .iter()
            .any(|p| p.source() == FeatureSource::Request(explicit)
                && p.role() == FeatureRole::Optional));

        let mut source = input("AA", crate::script::LATIN);
        source.set_direction(Direction::TopToBottom);
        let vertical_plan = ShapePlan::new(
            font,
            Direction::TopToBottom,
            Some(crate::script::LATIN),
            None,
            &canonical,
        );
        let mut queried = Vec::new();
        let vertical = font
            .shape_vertical_with_metrics(
                &vertical_plan,
                source,
                &reordered,
                crate::VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |glyph| {
                    queried.push(glyph);
                    Ok::<_, core::convert::Infallible>(crate::VerticalNominalMetrics::new(10, 0, 0))
                },
            )
            .unwrap();
        assert_eq!(vertical.glyph_infos().len(), 1);
        assert_eq!(vertical.glyph_infos()[0].glyph_id, u32::from(replacement));
        assert!(!queried.is_empty());
        assert!(queried.iter().all(|glyph| *glyph == u32::from(replacement)));
        let receipt = vertical.component_receipt().unwrap().unwrap();
        assert_eq!(
            receipt.default_features()[0].disposition(),
            DefaultFeatureDisposition::Accepted
        );
        assert!(receipt
            .ligatures()
            .next()
            .unwrap()
            .provenance()
            .iter()
            .any(|p| p.source() == FeatureSource::Request(explicit)
                && p.role() == FeatureRole::Optional));
    });
}

#[test]
fn reused_lookup_in_distinct_stages_observes_only_actual_stage_sources() {
    let mut a = 0;
    let mut b = 0;
    with_font(|font| {
        a = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16;
        b = font.get_nominal_glyph('B' as u32).unwrap().to_u32() as u16;
    });
    with_gsub(&shared_lookup_table(&[*b"liga", *b"rvrn"], a, b), |font| {
        let result = font.shape(input("AA", crate::script::LATIN), &[]);
        assert_eq!(result.len(), 1);
        let receipt = result.component_receipt().unwrap().unwrap();
        let relation = receipt.ligatures().next().unwrap();
        assert!(relation
            .provenance()
            .iter()
            .any(|p| p.tag() == hb_tag_t::new(b"rvrn")));
        assert!(!relation
            .provenance()
            .iter()
            .any(|p| p.tag() == hb_tag_t::new(b"liga")));

        let disabled = Feature::new(hb_tag_t::new(b"rvrn"), 0, ..);
        let result = font.shape(input("AA", crate::script::LATIN), &[disabled]);
        assert_eq!(result.len(), 1);
        let receipt = result.component_receipt().unwrap().unwrap();
        let relation = receipt.ligatures().next().unwrap();
        assert!(relation
            .provenance()
            .iter()
            .any(|p| p.tag() == hb_tag_t::new(b"liga")));
        assert!(!relation
            .provenance()
            .iter()
            .any(|p| p.tag() == hb_tag_t::new(b"rvrn")));
    });
}

#[test]
fn constructor_bounds_execute_included_empty_unbounded_and_excluded_start_requests() {
    use core::ops::Bound::{Excluded, Included};
    with_ligature(b"liga", |font, _| {
        assert_eq!(font.get_nominal_glyph('A' as u32).unwrap().to_u32(), 35);
        assert_eq!(font.get_nominal_glyph('B' as u32).unwrap().to_u32(), 36);
        let tag = hb_tag_t::new(b"liga");
        for (feature, expected) in [
            (Feature::new(tag, 0, ..=0), alloc::vec![35, 36, 35]),
            (Feature::new(tag, 0, ..1), alloc::vec![35, 36, 35]),
            (Feature::new(tag, 0, 1..=1), alloc::vec![35, 35, 36]),
            (
                Feature::new(tag, 0, (Excluded(0), Excluded(2))),
                alloc::vec![35, 35, 36],
            ),
            (Feature::new(tag, 0, 0..0), alloc::vec![36, 36]),
            (
                Feature::new(tag, 0, (Included(3), Excluded(1))),
                alloc::vec![36, 36],
            ),
            (Feature::new(tag, 0, ..), alloc::vec![35, 35, 35, 35]),
            (Feature::new(tag, 0, 3..), alloc::vec![36, 35, 35]),
            (
                Feature::new(tag, 0, ..=usize::MAX),
                alloc::vec![35, 35, 35, 35],
            ),
        ] {
            let result = font.shape(input("AAAA", crate::script::LATIN), &[feature]);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(
                result
                    .glyph_infos()
                    .iter()
                    .map(|g| g.glyph_id)
                    .collect::<Vec<_>>(),
                expected
            );
            assert!(result.glyph_positions().iter().all(|p| p.x_advance == 1000
                && p.x_offset == 0
                && p.y_offset == 0
                && p.y_advance == 0));
        }
    });
}

#[test]
fn constructor_bounds_saturate_to_existing_global_sentinel_without_overflow() {
    use core::ops::Bound::{Excluded, Unbounded};
    with_ligature(b"liga", |font, _| {
        let tag = hb_tag_t::new(b"liga");
        let max = u32::MAX as usize;
        for (feature, expected) in [
            // The complete sentinel encoding is global, including cluster MAX.
            (Feature::new(tag, 0, ..), alloc::vec![35, 35, 35, 35]),
            (
                Feature::new(tag, 0, ..=usize::MAX),
                alloc::vec![35, 35, 35, 35],
            ),
            // A nonzero-start sentinel-ended range excludes cluster MAX.
            (Feature::new(tag, 0, max - 1..=max), alloc::vec![35, 35, 36]),
            (
                Feature::new(tag, 0, max - 1..=usize::MAX),
                alloc::vec![35, 35, 36],
            ),
            (Feature::new(tag, 0, max..), alloc::vec![36, 36]),
            (
                Feature::new(tag, 0, (Excluded(usize::MAX), Unbounded)),
                alloc::vec![36, 36],
            ),
        ] {
            let mut value = UnicodeBuffer::new();
            for cluster in [u32::MAX - 1, u32::MAX - 1, u32::MAX, u32::MAX] {
                value.add('A', cluster);
            }
            value.set_script(crate::script::LATIN);
            value.set_direction(Direction::LeftToRight);
            let result = font.shape(value, &[feature]);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(
                result
                    .glyph_infos()
                    .iter()
                    .map(|g| g.glyph_id)
                    .collect::<Vec<_>>(),
                expected
            );
        }
    });
}
