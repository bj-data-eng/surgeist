use super::*;
use crate::hb::completion_fixture::{with_font, with_tables};
use crate::hb::unicode::GeneralCategory;
use crate::{
    script, Direction, JoiningInputError, JoiningSource, JoiningSourceError, ShapingStatus,
    UnicodeBuffer,
};
use alloc::string::String;
use alloc::vec::Vec;

fn input(text: &str) -> UnicodeBuffer {
    let mut value = UnicodeBuffer::new();
    for c in text.chars() {
        // Cluster aliases do not authorize semantic metadata.
        value.add(c, 0);
    }
    value.set_script(script::LATIN);
    value.set_direction(Direction::LeftToRight);
    value
}

fn cmap(groups: &[(u32, u32)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(0u16.to_be_bytes());
    bytes.extend(1u16.to_be_bytes());
    bytes.extend(3u16.to_be_bytes());
    bytes.extend(10u16.to_be_bytes());
    bytes.extend(12u32.to_be_bytes());
    bytes.extend(12u16.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((16 + groups.len() as u32 * 12).to_be_bytes());
    bytes.extend(0u32.to_be_bytes());
    bytes.extend((groups.len() as u32).to_be_bytes());
    for &(cp, gid) in groups {
        bytes.extend(cp.to_be_bytes());
        bytes.extend(cp.to_be_bytes());
        bytes.extend(gid.to_be_bytes());
    }
    bytes
}

#[test]
fn eligible_controls_keep_actual_nominal_glyphs_and_symbol_properties() {
    // The required C0/C1 inventories, independently of admission's predicate.
    let points: Vec<_> = (0..=31)
        .chain(127..=159)
        .filter(|cp| !matches!(cp, 9 | 10 | 13))
        .collect();
    assert_eq!(points.len(), 62);
    let mappings: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, &cp)| (cp, 35 + i as u32))
        .collect();
    let table = cmap(&mappings);
    with_tables(&[(*b"cmap", &table)], |font| {
        let text: String = points
            .iter()
            .map(|&cp| char::from_u32(cp).unwrap())
            .collect();
        let mut value = input(&text);
        let indices: Vec<_> = (0..62).collect();
        value.set_ordinary_symbol_positions(&indices).unwrap();
        assert_eq!(
            value.0.info[..value.len()]
                .iter()
                .map(|i| i.glyph_id)
                .collect::<Vec<_>>(),
            points
        );
        let actual = font.shape(value, &[]);
        assert_eq!(actual.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            actual
                .glyph_infos()
                .iter()
                .map(|i| i.glyph_id)
                .collect::<Vec<_>>(),
            (35..97).collect::<Vec<_>>()
        );
        assert!(actual
            .glyph_infos()
            .iter()
            .all(|i| i.general_category() == GeneralCategory::OTHER_SYMBOL
                && !i.is_default_ignorable()
                && i.modified_combining_class() == 0));
    });
}

#[test]
fn invalid_metadata_is_typed_atomic_and_excludes_noneligible_scalars() {
    with_font(|font| {
        for (positions, error) in [
            (&[3][..], OrdinarySymbolInputError::SourceIndex),
            (&[1, 0][..], OrdinarySymbolInputError::SourceOrder),
            (&[0, 0][..], OrdinarySymbolInputError::SourceOrder),
            (&[2][..], OrdinarySymbolInputError::SourceCharacter),
        ] {
            let mut value = input("\u{1}\u{2}A");
            value.set_ordinary_symbol_positions(&[0]).unwrap();
            assert_eq!(value.set_ordinary_symbol_positions(positions), Err(error));
            let actual = font.shape(value, &[]);
            assert_eq!(actual.shaping_status(), ShapingStatus::Complete);
            assert_eq!(
                actual.glyph_infos()[0].general_category(),
                GeneralCategory::OTHER_SYMBOL
            );
        }
        for c in ['\t', '\n', '\r', 'A', '\u{200d}', '\u{2065}'] {
            assert_eq!(
                input(&alloc::format!("{c}")).set_ordinary_symbol_positions(&[0]),
                Err(OrdinarySymbolInputError::SourceCharacter)
            );
        }
    });
}

#[test]
fn append_refuses_stale_binding_and_both_clear_paths_dispose_it() {
    with_font(|font| {
        let mut value = input("\u{1}");
        value.set_ordinary_symbol_positions(&[0]).unwrap();
        value.add('A', 0);
        let actual = font.shape(value, &[]);
        assert_eq!(
            actual.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::OrdinarySymbolInput)
        );
        let mut value = actual.clear();
        value.push_str("A");
        value.set_direction(Direction::LeftToRight);
        value.set_script(script::LATIN);
        assert_eq!(
            font.shape(value, &[]).shaping_status(),
            ShapingStatus::Complete
        );
        let mut value = input("\u{1}");
        value.set_ordinary_symbol_positions(&[0]).unwrap();
        value.clear();
        value.push_str("A");
        value.set_direction(Direction::LeftToRight);
        value.set_script(script::LATIN);
        assert_eq!(
            font.shape(value, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}

#[test]
fn joining_sources_require_both_codepoint_and_presentation_and_revalidate_mutation() {
    with_font(|font| {
        let native =
            JoiningSource::analyze("\u{1}", script::LATIN, Direction::LeftToRight, &[0]).unwrap();
        let ordinary =
            JoiningSource::analyze("\u{1}", script::LATIN, Direction::LeftToRight, &[]).unwrap();
        for profiled in [false, true] {
            let mut value = input("\u{1}");
            if profiled {
                value.set_ordinary_symbol_positions(&[0]).unwrap();
            }
            let wrong = if profiled { &ordinary } else { &native };
            assert_eq!(
                value.set_joining_sources(wrong, &[Some(0)], wrong, &[0]),
                Err(JoiningInputError::SourcePresentation)
            );
            let right = if profiled { &native } else { &ordinary };
            value
                .set_joining_sources(right, &[Some(0)], right, &[0])
                .unwrap();
            value
                .set_ordinary_symbol_positions(if profiled { &[] } else { &[0] })
                .unwrap();
            assert_eq!(
                font.shape(value, &[]).shaping_status(),
                ShapingStatus::Refused(ShapingRefusal::JoiningInput)
            );
        }
        assert_eq!(
            JoiningSource::analyze("A", script::LATIN, Direction::LeftToRight, &[0]).unwrap_err(),
            JoiningSourceError::OrdinarySymbol(OrdinarySymbolInputError::SourceCharacter)
        );
    });
}

#[test]
fn normalization_preserves_scalar_profile_with_aliased_clusters_and_joint_sources() {
    with_font(|font| {
        let text = "\u{1}\u{212b}\u{301}\u{327}";
        let source =
            JoiningSource::analyze(text, script::LATIN, Direction::LeftToRight, &[0]).unwrap();
        let mut value = input(text);
        value.set_ordinary_symbol_positions(&[0]).unwrap();
        value
            .set_joining_sources(
                &source,
                &[Some(0), Some(1), Some(2), Some(3)],
                &source,
                &[0, 1, 2, 3],
            )
            .unwrap();
        let actual = font.shape(value, &[]);
        assert_eq!(actual.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            actual
                .glyph_infos()
                .iter()
                .filter(|i| i.is_ordinary_symbol())
                .count(),
            1
        );
        assert_eq!(
            actual.glyph_infos()[0].general_category(),
            GeneralCategory::OTHER_SYMBOL
        );
        assert!(actual
            .glyph_infos()
            .iter()
            .skip(1)
            .all(|i| !i.is_ordinary_symbol()));
    });
}

#[test]
fn profile_work_is_charged_before_accepting_properties() {
    let mut value = input("\u{1}");
    value.set_ordinary_symbol_positions(&[0]).unwrap();
    value.0.max_ops = 0;
    assert!(!begin(&mut value.0));
    assert_eq!(
        value.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
    );
}

#[test]
fn failed_work_admission_preserves_existing_profile_and_status() {
    let mut value = input("\u{1}");
    value.set_ordinary_symbol_positions(&[0]).unwrap();
    value.0.max_ops = 0;
    assert_eq!(
        value.set_ordinary_symbol_positions(&[]),
        Err(OrdinarySymbolInputError::WorkLimit)
    );
    assert!(value.0.info[0].is_ordinary_symbol());
    assert_eq!(value.0.shaping_status(), ShapingStatus::Complete);
}

#[test]
fn admission_work_boundary_and_growth_failure_are_atomic() {
    let mut value = input("\u{1}\u{2}");
    value.set_ordinary_symbol_positions(&[0]).unwrap();
    // Two rows plus two inspections/writes for the one requested position.
    value.0.max_ops = 3;
    assert_eq!(
        value.set_ordinary_symbol_positions(&[1]),
        Err(OrdinarySymbolInputError::WorkLimit)
    );
    assert!(value.0.info[0].is_ordinary_symbol());
    assert!(!value.0.info[1].is_ordinary_symbol());
    assert_eq!(value.0.max_ops, 3);
    value.0.max_ops = 4;
    value.set_ordinary_symbol_positions(&[1]).unwrap();
    assert_eq!(value.0.max_ops, 0);
    assert!(!value.0.info[0].is_ordinary_symbol());
    assert!(value.0.info[1].is_ordinary_symbol());

    // Exercise the logical length guard before any impossible row access or
    // allocation; the real rows and prior admitted profile remain unchanged.
    value.0.len = hb_buffer_t::MAX_LEN_DEFAULT + 1;
    assert_eq!(
        value.set_ordinary_symbol_positions(&[]),
        Err(OrdinarySymbolInputError::GrowthLimit)
    );
    value.0.len = 2;
    assert!(!value.0.info[0].is_ordinary_symbol());
    assert!(value.0.info[1].is_ordinary_symbol());
    assert_eq!(value.0.shaping_status(), ShapingStatus::Complete);
}

#[test]
fn final_input_rebinding_and_empty_removal_restore_valid_shaping() {
    with_font(|font| {
        for remove in [false, true] {
            let mut value = input("\u{1}");
            value.set_ordinary_symbol_positions(&[0]).unwrap();
            value.push_str("\u{2}");
            value
                .set_ordinary_symbol_positions(if remove { &[] } else { &[0, 1] })
                .unwrap();
            let actual = font.shape(value, &[]);
            assert_eq!(actual.shaping_status(), ShapingStatus::Complete);
            assert!(actual.glyph_infos().iter().all(|info| {
                info.general_category()
                    == if remove {
                        GeneralCategory::CONTROL
                    } else {
                        GeneralCategory::OTHER_SYMBOL
                    }
            }));
        }
    });
}

#[test]
fn script_specific_shapers_keep_the_nominal_control_symbol_and_following_mark() {
    let points: Vec<_> = (0..=31)
        .chain(127..=159)
        .filter(|cp| !matches!(cp, 9 | 10 | 13))
        .collect();
    let mappings: Vec<_> = points
        .iter()
        .map(|&cp| (cp, 35))
        .chain([(0x301, 36)])
        .collect();
    let table = cmap(&mappings);
    with_tables(&[(*b"cmap", &table)], |font| {
        for &point in &points {
            let text = alloc::format!("{}\u{301}", char::from_u32(point).unwrap());
            for script in [
                script::ARABIC,
                script::DEVANAGARI,
                script::KHMER,
                script::MYANMAR,
                script::BALINESE,
                script::HANGUL,
            ] {
                for direction in [Direction::LeftToRight, Direction::RightToLeft] {
                    let source = JoiningSource::analyze(&text, script, direction, &[0]).unwrap();
                    let mut value = input(&text);
                    value.set_script(script);
                    value.set_direction(direction);
                    value.set_ordinary_symbol_positions(&[0]).unwrap();
                    value
                        .set_joining_sources(&source, &[Some(0), Some(1)], &source, &[0, 1])
                        .unwrap();
                    let actual = font.shape(value, &[]);
                    assert_eq!(
                        actual.shaping_status(),
                        ShapingStatus::Complete,
                        "{script:?} {direction:?}"
                    );
                    let glyphs: Vec<_> = actual.glyph_infos().iter().map(|i| i.glyph_id).collect();
                    assert!(glyphs.contains(&35), "{script:?} {direction:?}: {glyphs:?}");
                    assert!(glyphs.contains(&36), "{script:?} {direction:?}: {glyphs:?}");
                    assert_eq!(
                        actual
                            .glyph_infos()
                            .iter()
                            .filter(|i| i.is_ordinary_symbol())
                            .count(),
                        1
                    );
                }
            }
        }
    });
}
