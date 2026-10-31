//! Real fixed-reference operands for preserved TAB Cell width qualification.
//!
//! Intended cfg(test) owner: a private child of test_fonts/combined_width.rs.
//! Uses that owner's existing SFNT assembly and unique-family name writer.
//! Licensed Ahem outlines, cmap, hhea and nonidentity name payloads are retained;
//! preserve tests/fixtures/fonts/ahem/{PROVENANCE.md,COPYING,UPSTREAM-README}.
//! Ahem is public domain with CC0 fallback. New GSUB data is repository-authored.
//!
//! At UPEM1000, normal SPACE3/ZERO18/A35 each advance1000; B36 advances500.
//! Each actual hwid/twid/qwid plan replaces all three first operands with
//! B36/R58/period16 (500/400/300), leaving original B36 UNACTED. Equal output
//! glyphs do not turn that B source root into successful-action coverage.
//!
//! Font-operand derivation for a later real A TAB A Cell at size10: a specified
//! SPACE10 and tab-size2 supply the fixed period20 before artificial trials.
//! Candidate A advances10/5/4/3 put the second A at the same pen20, giving
//! widths30/25/24/23. The qualified narrowest packet would need scale10/23.
//! This font-only helper does NOT execute TAB, issue a TAB-layout proof, assert
//! Cell selection/compression, or infer trial counts from winning glyphs.

#![cfg(test)]

use super::{AHEM, fixture, layout_table, single_substitution};
use crate::test_fonts::{table, u16_at};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
};
use surgeist_shaper::{
    Direction, Feature, FeatureSource, FontRef, ShaperData, ShapingStatus, Tag, UnicodeBuffer,
    WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
};

pub(crate) const COMBINED_TAB_WIDTH_FAMILY: &str = "Surgeist Combined Tab Width Ahem";

/// A real three-tag derivative, selectable beside original and combined Ahem.
/// The existing owner supplies the same retained cmap, outlines and hmtx;
/// only this family's identity and authored GSUB routing differ.
pub(crate) fn combined_tab_width_ahem() -> Vec<u8> {
    let features = [(*b"hwid", 36_u16), (*b"twid", 58), (*b"qwid", 16)]
        .into_iter()
        .map(|(tag, glyph)| (tag, 1, single_substitution(&[3, 18, 35], &[glyph; 3])))
        .collect();
    fixture(COMBINED_TAB_WIDTH_FAMILY, layout_table(*b"DFLT", features))
}

#[derive(Clone, Copy)]
struct WidthPlan {
    tag: [u8; 4],
    feature: WidthFeature,
    glyph: u32,
    advance: i32,
}

fn packet(bytes: &[u8], direction: Direction, plan: Option<WidthPlan>) {
    let font = FontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    let mut input = UnicodeBuffer::new();
    for (ordinal, ch) in " 0AB".chars().enumerate() {
        input.add(ch, u32::try_from(ordinal).unwrap());
    }
    input.set_direction(direction);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let request = plan.map(|plan| Feature::new(Tag::new(&plan.tag), 1, ..));
    if let Some(plan) = plan {
        input.observe_width_feature(plan.feature);
    }
    let features = request.into_iter().collect::<Vec<_>>();
    let output = shaper.shape(input, &features);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert_eq!(output.glyph_infos().len(), 4);
    assert_eq!(output.glyph_positions().len(), 4);
    let mut actual = output
        .glyph_infos()
        .iter()
        .zip(output.glyph_positions())
        .map(|(info, pos)| {
            (
                info.cluster,
                info.glyph_id,
                pos.x_advance,
                pos.y_advance,
                pos.x_offset,
                pos.y_offset,
            )
        })
        .collect::<Vec<_>>();
    // Retain actual source ordinals, independent of the provider's visual order.
    // Both horizontal directions must preserve the same literal font operands.
    actual.sort_unstable_by_key(|row| row.0);
    let expected = if let Some(plan) = plan {
        [
            (0, plan.glyph, plan.advance, 0, 0, 0),
            (1, plan.glyph, plan.advance, 0, 0, 0),
            (2, plan.glyph, plan.advance, 0, 0, 0),
            (3, 36, 500, 0, 0, 0),
        ]
    } else {
        [
            (0, 3, 1000, 0, 0, 0),
            (1, 18, 1000, 0, 0, 0),
            (2, 35, 1000, 0, 0, 0),
            (3, 36, 500, 0, 0, 0),
        ]
    };
    assert_eq!(actual, expected);
    if let Some(plan) = plan {
        let receipt = output.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), plan.feature);
        assert_eq!(receipt.input_len(), 4);
        assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
        for ordinal in 0..4 {
            let input = receipt.input(ordinal).unwrap();
            if ordinal < 3 {
                assert_eq!(input.outcome(), WidthInputOutcome::Covered);
                assert!(input.provenance().iter().any(|entry| {
                    entry.tag() == Tag::new(&plan.tag)
                        && entry.source() == FeatureSource::Request(request.unwrap())
                }));
            } else {
                assert_eq!(input.outcome(), WidthInputOutcome::NoAction);
                assert!(input.provenance().is_empty());
            }
        }
        assert!(receipt.input(4).is_none());
    } else {
        assert!(output.width_feature_receipt().unwrap().is_none());
    }
}

fn name_payload(bytes: &[u8], record: usize) -> &[u8] {
    let strings = usize::from(u16_at(bytes, 4));
    let start = strings + usize::from(u16_at(bytes, record + 10));
    let len = usize::from(u16_at(bytes, record + 8));
    &bytes[start..start + len]
}

/// Qualify the EXACT resource used by a consumer. A native BASE/vmtx wrapper
/// may be applied first: added native tables/head checksum do not weaken these
/// real cmap, horizontal operands, retained resources or action requirements.
pub(crate) fn qualify_combined_tab_width(bytes: &[u8]) {
    let parsed = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(u16_at(table(bytes, *b"head"), 18), 1000);
    for (ch, glyph) in [(' ', 3), ('0', 18), ('A', 35), ('B', 36)] {
        assert_eq!(parsed.charmap().map(ch), Some(GlyphId::new(glyph)));
    }
    let metrics = parsed.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (glyph, advance) in [
        (3, 1000),
        (18, 1000),
        (35, 1000),
        (36, 500),
        (58, 400),
        (16, 300),
    ] {
        assert_eq!(
            u16_at(table(bytes, *b"hmtx"), usize::try_from(glyph).unwrap() * 4),
            advance
        );
        assert_eq!(
            metrics.advance_width(GlyphId::new(glyph)),
            Some(f32::from(advance))
        );
    }
    for tag in [
        *b"glyf", *b"loca", *b"maxp", *b"cmap", *b"hhea", *b"OS/2", *b"post",
    ] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    let source = table(AHEM, *b"name");
    let names = table(bytes, *b"name");
    assert_eq!((u16_at(source, 0), u16_at(names, 0)), (0, 0));
    assert_eq!(u16_at(source, 2), u16_at(names, 2));
    for index in 0..usize::from(u16_at(source, 2)) {
        let row = 6 + index * 12;
        assert_eq!(&source[row..row + 8], &names[row..row + 8]);
        let name = u16_at(source, row + 6);
        // Identity fields are renamed by the existing licensed-fixture owner.
        // All other payloads include copyright/vendor/license/URL records.
        if let Some(value) = super::identity_name(name, COMBINED_TAB_WIDTH_FAMILY) {
            let expected =
                super::encoded_identity(u16_at(source, row), u16_at(source, row + 2), &value);
            assert_eq!(name_payload(names, row), expected.as_slice());
        } else {
            assert_eq!(name_payload(names, row), name_payload(source, row));
        }
    }
    for direction in [Direction::LeftToRight, Direction::RightToLeft] {
        packet(bytes, direction, None);
        for plan in [
            WidthPlan {
                tag: *b"hwid",
                feature: WidthFeature::HalfWidth,
                glyph: 36,
                advance: 500,
            },
            WidthPlan {
                tag: *b"twid",
                feature: WidthFeature::ThirdWidth,
                glyph: 58,
                advance: 400,
            },
            WidthPlan {
                tag: *b"qwid",
                feature: WidthFeature::QuarterWidth,
                glyph: 16,
                advance: 300,
            },
        ] {
            packet(bytes, direction, Some(plan));
        }
    }
}

#[test]
fn tab_reference_font_has_real_metric_changing_width_actions_and_unacted_b_in_both_directions() {
    let bytes = combined_tab_width_ahem();
    qualify_combined_tab_width(&bytes);
}
