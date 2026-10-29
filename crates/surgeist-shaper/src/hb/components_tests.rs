//! New-operation qualification from authored SFNT tables and nominal records.
//! No executable preimplementation RED is claimed for the observer API.
use super::{font, input, make_plan, old_positions, qualify};
use crate::hb::completion_fixture::words;
use crate::{
    ComponentAttachmentKind, ComponentCoverage, ComponentPath, Direction, FeatureRole,
    ShapingRefusal, ShapingStatus, VerticalOriginReference,
};
use alloc::{vec, vec::Vec};
use core::convert::Infallible;

fn observed(text: &str, direction: Direction) -> crate::UnicodeBuffer {
    let mut value = input(text, direction);
    value.observe_components();
    value
}
fn ids(value: &crate::GlyphBuffer) -> Vec<u32> {
    value.glyph_infos().iter().map(|g| g.glyph_id).collect()
}
fn multiple(from: u16, to: &[u16]) -> (u16, Vec<u8>) {
    let mut data = words(&[
        1,
        u16::try_from(10 + 2 * to.len()).unwrap(),
        1,
        8,
        u16::try_from(to.len()).unwrap(),
    ]);
    data.extend(words(to));
    data.extend(words(&[1, 1, from]));
    (2, data)
}
#[test]
fn component_observer_ordinary_and_plan_fronts_keep_raw_geometry_and_actual_ordinals() {
    font::with_font(true, false, None, None, &[], |font| {
        qualify(font, true, false);
        let ordinary = font.shape(input("AB", Direction::LeftToRight), &[]);
        assert!(ordinary.component_receipt().unwrap().is_none());
        assert_eq!(ordinary.0.components.bytes(), Some(0));
        for before in [false, true] {
            let mut value = crate::UnicodeBuffer::new();
            if before {
                value.observe_components();
            }
            value.add('A', 777);
            value.add('B', 777);
            if !before {
                value.observe_components();
            }
            value.set_pre_context("RR");
            value.set_post_context("RR");
            value.set_direction(Direction::LeftToRight);
            value.set_script(crate::script::LATIN);
            let plan = make_plan(font, &value, &[]);
            let result = font.shape_with_plan(&plan, value, &[]);
            assert_eq!(ids(&result), vec![u32::from(font::A), u32::from(font::B)]);
            assert_eq!(old_positions(&result), old_positions(&ordinary));
            let receipt = result.component_receipt().unwrap().unwrap();
            assert_eq!(receipt.input_len(), 2);
            assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
            assert_eq!(receipt.components(0), Some(&[0][..]));
            assert_eq!(receipt.components(1), Some(&[1][..]));
            assert_eq!(receipt.components(2), None);
            assert_eq!(receipt.ligatures().count(), 0);
            assert_eq!(receipt.attachments().count(), 0);
            assert!(receipt.deleted_input_positions().is_empty());
        }
    });
}
#[test]
fn component_observer_ligature_final_descendants_survive_multiple_rewrite_and_deletion() {
    for outputs in [vec![font::A, font::B], vec![]] {
        let gsub = font::layout_table(b"rlig", &[font::ligature(), multiple(font::R, &outputs)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = font.shape(observed("AB", Direction::LeftToRight), &[]);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(
                ids(&result),
                outputs.iter().map(|&g| u32::from(g)).collect::<Vec<_>>()
            );
            let receipt = result.component_receipt().unwrap().unwrap();
            let row = receipt
                .ligatures()
                .next()
                .expect("authored A+B -> R lookup must actually ligate");
            assert_eq!(row.components(), &[0, 1]);
            assert_eq!(
                row.glyph_indices(),
                if outputs.is_empty() {
                    &[][..]
                } else {
                    &[0, 1][..]
                }
            );
            assert!(row
                .provenance()
                .iter()
                .any(|p| p.tag() == crate::Tag::new(b"rlig") && p.role() == FeatureRole::Required));
            if outputs.is_empty() {
                assert_eq!(receipt.deleted_input_positions(), &[0, 1]);
            } else {
                assert_eq!(receipt.components(0), Some(&[0, 1][..]));
                assert_eq!(receipt.components(1), Some(&[0, 1][..]));
                assert!(receipt.deleted_input_positions().is_empty());
            }
        });
    }
}
#[test]
fn component_observer_skipped_mark_is_not_in_consumed_ligature_set_and_attachment_is_real() {
    let mut gsub = font::layout_table(b"rlig", &[font::ligature()]);
    // LookupList offset is the real header field; set lookup0 IgnoreMarks.
    let list = u16::from_be_bytes(gsub[8..10].try_into().unwrap()) as usize;
    let lookup = list + u16::from_be_bytes(gsub[list + 2..list + 4].try_into().unwrap()) as usize;
    gsub[lookup + 2..lookup + 4].copy_from_slice(&8u16.to_be_bytes());
    let gpos = font::layout_table(
        b"mark",
        &[font::mark(font::R, font::MARK, (500, 700), (10, 20), 4)],
    );
    font::with_font(true, false, Some(&gsub), Some(&gpos), &[], |font| {
        assert_eq!(
            font.get_nominal_glyph(0x0300).unwrap().to_u32(),
            u32::from(font::MARK)
        );
        let result = font.shape(observed("A\u{300}B", Direction::LeftToRight), &[]);
        assert_eq!(
            ids(&result),
            vec![u32::from(font::R), u32::from(font::MARK)]
        );
        assert_eq!(result.glyph_positions()[0].x_advance, 1201);
        assert_eq!(result.glyph_positions()[1].x_advance, 0);
        assert_eq!(result.glyph_positions()[1].x_offset, -711);
        assert_eq!(result.glyph_positions()[1].y_offset, 680);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.components(0), Some(&[0, 2][..]));
        assert_eq!(receipt.components(1), Some(&[1][..]));
        let lig = receipt.ligatures().next().unwrap();
        assert_eq!(lig.components(), &[0, 2]);
        assert_eq!(lig.glyph_indices(), &[0]);
        let mark = receipt.attachments().next().unwrap();
        assert_eq!(mark.kind(), ComponentAttachmentKind::Mark);
        assert_eq!(mark.dependent_components(), &[1]);
        assert_eq!(mark.base_components(), &[0, 2]);
    });
}
#[test]
fn component_observer_actual_gpos_mark_cursive_and_pair_have_distinct_constraints() {
    for (lookup, text, kind) in [
        (
            font::mark(font::A, font::MARK, (500, 700), (10, 20), 4),
            "A\u{300}",
            Some(ComponentAttachmentKind::Mark),
        ),
        (
            font::cursive(),
            "AB",
            Some(ComponentAttachmentKind::Cursive),
        ),
        (font::pair(), "AB", None),
    ] {
        let gpos = font::layout_table(
            if kind == Some(ComponentAttachmentKind::Mark) {
                b"mark"
            } else if kind == Some(ComponentAttachmentKind::Cursive) {
                b"curs"
            } else {
                b"kern"
            },
            &[lookup],
        );
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let result = font.shape(observed(text, Direction::LeftToRight), &[]);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            let receipt = result.component_receipt().unwrap().unwrap();
            let kinds = receipt
                .attachments()
                .map(|row| row.kind())
                .collect::<Vec<_>>();
            assert_eq!(kinds, kind.into_iter().collect::<Vec<_>>());
            assert_eq!(receipt.ligatures().count(), 0);
            if kind.is_none() {
                assert_eq!(result.glyph_positions()[0].x_advance, 1099);
                assert_eq!(result.glyph_positions()[1].x_advance, 1088);
            }
        });
    }
}
#[test]
fn component_observer_supplied_vertical_and_reversed_output_keep_final_correspondence() {
    let gsub = font::layout_table(b"rlig", &[font::ligature()]);
    font::with_font(true, true, Some(&gsub), None, &[], |font| {
        let value = observed("ABR", Direction::TopToBottom);
        let plan = make_plan(font, &value, &[]);
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                value,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |gid| Ok::<_, Infallible>(font::metrics(font, gid)),
            )
            .unwrap();
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.components(0), Some(&[0, 1][..]));
        assert_eq!(receipt.components(1), Some(&[2][..]));
        assert_eq!(receipt.ligatures().next().unwrap().glyph_indices(), &[0]);
        let reversed = font.shape(observed("ABR", Direction::RightToLeft), &[]);
        let receipt = reversed.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.components(0), Some(&[2][..]));
        assert_eq!(receipt.components(1), Some(&[1][..]));
        assert_eq!(receipt.components(2), Some(&[0][..]));
        let mut clean = result.clear();
        clean.push_str("A");
        clean.set_direction(Direction::LeftToRight);
        clean.set_script(crate::script::LATIN);
        assert!(font
            .shape(clean, &[])
            .component_receipt()
            .unwrap()
            .is_none());
    });
}
#[test]
fn component_observer_buffer_clear_and_resource_refusal_preserve_lifecycle() {
    font::with_font(true, false, None, None, &[], |font| {
        let mut value = observed("AB", Direction::LeftToRight);
        value.clear();
        value.push_str("A");
        value.set_direction(Direction::LeftToRight);
        value.set_script(crate::script::LATIN);
        assert!(font
            .shape(value, &[])
            .component_receipt()
            .unwrap()
            .is_none());
        let mut value = observed("A", Direction::LeftToRight);
        value.0.refuse(ShapingRefusal::ContextLimit);
        let failed = font.shape(value, &[]);
        assert!(matches!(
            failed.component_receipt(),
            Err(ShapingRefusal::ContextLimit)
        ));
        let mut clean = failed.clear();
        clean.observe_components();
        clean.push_str("AB");
        clean.set_direction(Direction::LeftToRight);
        clean.set_script(crate::script::LATIN);
        assert_eq!(
            font.shape(clean, &[])
                .component_receipt()
                .unwrap()
                .unwrap()
                .input_len(),
            2
        );
        // Exercise actual observer initialization guards at their owning boundary.
        let mut value = observed("AB", Direction::LeftToRight);
        let plan = make_plan(font, &value, &[]);
        value.0.max_len = 0;
        assert!(!crate::hb::components::begin(&mut value.0, &plan));
        assert_eq!(
            value.0.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
        let mut value = observed("AB", Direction::LeftToRight);
        value.0.max_ops = 0;
        assert!(!crate::hb::components::begin(&mut value.0, &plan));
        assert_eq!(
            value.0.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::WorkLimit)
        );
    });
}
#[test]
fn component_observer_actual_aat_substitution_is_unqualified_but_irrelevant_table_is_not() {
    let morx = font::morx();
    font::with_font(true, false, None, None, &[(*b"morx", &morx)], |font| {
        let result = font.shape(observed("AB", Direction::LeftToRight), &[]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            result.component_receipt().unwrap().unwrap().coverage(),
            ComponentCoverage::Unqualified(ComponentPath::AatSubstitution)
        );
    });
    let gsub = font::layout_table(b"rlig", &[font::ligature()]);
    font::with_font(
        true,
        false,
        Some(&gsub),
        None,
        &[(*b"morx", &morx)],
        |font| {
            let result = font.shape(observed("AB", Direction::TopToBottom), &[]);
            assert_eq!(
                result.component_receipt().unwrap().unwrap().coverage(),
                ComponentCoverage::Complete
            );
        },
    );
}

#[test]
fn component_observer_nested_context_keeps_required_parent_when_child_feature_is_disabled() {
    // DFLT: rlig -> lookup0 Context(A,B) -> lookup1 Ligature(A,B -> R).
    // The child also has an independent optional liga registration, explicitly
    // disabled. Its successful recursive application is authorized by rlig.
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, 2, 0, 1]));
    let mut features = words(&[2]);
    features.extend(*b"liga");
    features.extend(words(&[14]));
    features.extend(*b"rlig");
    features.extend(words(&[20, 0, 1, 1, 0, 1, 0]));
    let context = words(&[3, 2, 1, 14, 20, 0, 1, 1, 1, font::A, 1, 1, font::B]);
    let (_, ligature) = font::ligature();
    let mut lookups = words(&[2, 6, u16::try_from(14 + context.len()).unwrap(), 5, 0, 1, 8]);
    lookups.extend(context);
    lookups.extend(words(&[4, 0, 1, 8]));
    lookups.extend(ligature);
    let mut gsub = words(&[
        1,
        0,
        10,
        u16::try_from(10 + scripts.len()).unwrap(),
        u16::try_from(10 + scripts.len() + features.len()).unwrap(),
    ]);
    gsub.extend(scripts);
    gsub.extend(features);
    gsub.extend(lookups);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let disabled = crate::Feature::new(crate::Tag::new(b"liga"), 0, ..);
        let result = font.shape(observed("AB", Direction::LeftToRight), &[disabled]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(ids(&result), vec![u32::from(font::R)]);
        let receipt = result.component_receipt().unwrap().unwrap();
        let row = receipt.ligatures().next().unwrap();
        assert_eq!(row.components(), &[0, 1]);
        assert_eq!(row.glyph_indices(), &[0]);
        assert!(row
            .provenance()
            .iter()
            .any(|p| p.tag() == crate::Tag::new(b"rlig") && p.role() == FeatureRole::Required));
        assert!(!row
            .provenance()
            .iter()
            .any(|p| p.tag() == crate::Tag::new(b"liga")));
    });
}

#[test]
fn component_observer_normalization_is_not_ligation_and_fallback_marks_keep_dependence() {
    font::with_font(true, false, None, None, &[], |font| {
        let ordinary = font.shape(input("A\u{300}", Direction::LeftToRight), &[]);
        let result = font.shape(observed("A\u{300}", Direction::LeftToRight), &[]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            ids(&result),
            vec![u32::from(font::A), u32::from(font::MARK)]
        );
        assert_eq!(old_positions(&result), old_positions(&ordinary));
        assert_ne!(result.glyph_positions()[1].y_offset, 0);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.ligatures().count(), 0);
        let row = receipt.attachments().next().unwrap();
        assert_eq!(row.kind(), ComponentAttachmentKind::Mark);
        assert_eq!(row.dependent_components(), &[1]);
        assert_eq!(row.base_components(), &[0]);
        // A real canonical decomposition follows Unicode's fixed U+00C0 mapping.
        let decomposed = font.shape(observed("\u{00c0}", Direction::LeftToRight), &[]);
        let receipt = decomposed.component_receipt().unwrap().unwrap();
        assert_eq!(
            ids(&decomposed),
            vec![u32::from(font::A), u32::from(font::MARK)]
        );
        assert_eq!(receipt.components(0), Some(&[0][..]));
        assert_eq!(receipt.components(1), Some(&[0][..]));
        assert_eq!(receipt.ligatures().count(), 0);
    });
}

#[test]
fn component_observer_reordered_marks_and_generated_dotted_circle_have_exact_custody() {
    font::with_font(true, false, None, None, &[], |font| {
        let result = font.shape(observed("A\u{315}\u{300}", Direction::LeftToRight), &[]);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(
            ids(&result),
            vec![
                u32::from(font::A),
                u32::from(font::MARK),
                u32::from(font::MARK2)
            ]
        );
        assert_eq!(receipt.components(1), Some(&[2][..]));
        assert_eq!(receipt.components(2), Some(&[1][..]));
        let mut value = observed("\u{300}", Direction::LeftToRight);
        value.set_flags(crate::BufferFlags::BEGINNING_OF_TEXT);
        let result = font.shape(value, &[]);
        assert_eq!(
            ids(&result),
            vec![u32::from(font::R), u32::from(font::MARK)]
        );
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.generated(0), Some(true));
        assert_eq!(receipt.components(0), Some(&[][..]));
        assert_eq!(receipt.generated_context(0), Some(&[0][..]));
        assert_eq!(receipt.components(1), Some(&[0][..]));
    });
}
