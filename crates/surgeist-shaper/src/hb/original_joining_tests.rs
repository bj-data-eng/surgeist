use super::*;
use crate::hb::completion_fixture::{with_font, with_tables};
use crate::{script, BufferFlags, ShapingStatus};

fn source(text: &str, script: Script) -> JoiningSource {
    JoiningSource::analyze(text, script, Direction::LeftToRight, &[]).unwrap()
}
fn input(text: &str, script: Script) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    // Genuine colliding cluster values must never merge scalar identities.
    for ch in text.chars() {
        input.add(ch, 0);
    }
    input.set_script(script);
    input.set_direction(Direction::LeftToRight);
    input
}
fn bind(
    input: &mut UnicodeBuffer,
    original: &JoiningSource,
    map: &[Option<usize>],
    presentation: &JoiningSource,
) {
    let indices: Vec<_> = (0..input.len()).collect();
    input
        .set_joining_sources(original, map, presentation, &indices)
        .unwrap();
}

#[test]
fn full_bodies_keep_literal_joining_actions_and_transparent_body_has_no_external_cap() {
    let source = JoiningSource::analyze(
        "\u{628}\u{628}\u{628}",
        script::ARABIC,
        Direction::RightToLeft,
        &[],
    )
    .unwrap();
    assert_eq!(
        source
            .0
            .scalars
            .iter()
            .map(|s| s.action)
            .collect::<Vec<_>>(),
        [6, 4, 1]
    );
    let long = source_with_transparent_body(300);
    assert_eq!(long.0.scalars.first().unwrap().action, 6);
    assert_eq!(long.0.scalars.last().unwrap().action, 1);
    assert!(long.0.scalars[1..301]
        .iter()
        .all(|scalar| scalar.action == 7));
}

#[test]
fn ordinary_symbols_interrupt_arabic_joining_in_the_full_source_domain() {
    // BEH on either side of a nonjoining symbol has an isolated action. The
    // action values are the provider's existing OpenType-feature contract:
    // isol=0, init=6, fina=1, and no feature=7.
    for direction in [Direction::LeftToRight, Direction::RightToLeft] {
        let connected =
            JoiningSource::analyze("\u{628}\u{628}", script::ARABIC, direction, &[]).unwrap();
        assert_eq!(
            connected
                .0
                .scalars
                .iter()
                .map(|s| s.action)
                .collect::<Vec<_>>(),
            // Arabic runs execute in their native RTL order; an explicit LTR
            // input reverses the scalar order before assigning these actions.
            if direction == Direction::RightToLeft {
                [6, 1]
            } else {
                [1, 6]
            }
        );
        for point in (0..=31)
            .chain(127..=159)
            .filter(|cp| !matches!(cp, 9 | 10 | 13))
        {
            let text = alloc::format!("\u{628}{}\u{628}", char::from_u32(point).unwrap());
            let separated = JoiningSource::analyze(&text, script::ARABIC, direction, &[1]).unwrap();
            assert_eq!(
                separated
                    .0
                    .scalars
                    .iter()
                    .map(|s| s.action)
                    .collect::<Vec<_>>(),
                [0, 7, 0]
            );
            assert_eq!(separated.0.scalars[1].class, Class::U);

            // The only raw external-context consumer is the joining scan.
            // Every eligible Cc is already nonjoining there, so its raw context
            // spelling must terminate joining just like the selected symbol.
            let context = alloc::format!("\u{628}{}", char::from_u32(point).unwrap());
            let post_context = alloc::format!("{}\u{628}", char::from_u32(point).unwrap());
            let mut external = input("\u{628}", script::ARABIC);
            external.set_direction(direction);
            external.set_pre_context(&context);
            external.set_post_context(&post_context);
            super::super::ot_shape::prepare_joining_source(&mut external.0);
            ot_shaper_arabic::source_joining_actions(script::ARABIC, &mut external.0);
            assert_eq!(external.0.info[0].arabic_shaping_action(), 0);
            assert_eq!(external.0.shaping_status(), ShapingStatus::Complete);
        }
    }
}
fn source_with_transparent_body(count: usize) -> JoiningSource {
    JoiningSource::analyze(
        &alloc::format!("\u{628}{}\u{628}", "\u{61c}".repeat(count)),
        script::ARABIC,
        Direction::RightToLeft,
        &[],
    )
    .unwrap()
}

#[test]
fn admission_is_atomic_and_shape_revalidates_final_input_profile_and_clear() {
    with_font(|font| {
        let original = source("AB", script::LATIN);
        let mut value = input("AB", script::LATIN);
        bind(&mut value, &original, &[Some(0), Some(1)], &original);
        assert_eq!(
            value.set_joining_sources(&original, &[Some(1), Some(0)], &original, &[0, 1]),
            Err(JoiningInputError::SourceCharacter)
        );
        let shaped = font.shape(value, &[]);
        assert_eq!(shaped.shaping_status(), ShapingStatus::Complete);
        assert_eq!(shaped.glyph_infos().len(), 2);
        assert!(shaped
            .glyph_infos()
            .iter()
            .all(|glyph| glyph.unsafe_to_break()));
        let mut cleared = shaped.clear();
        cleared.push_str("A");
        cleared.set_script(script::LATIN);
        cleared.set_direction(Direction::LeftToRight);
        assert_eq!(
            font.shape(cleared, &[]).shaping_status(),
            ShapingStatus::Complete
        );
        for mutate in [0, 1] {
            let mut value = input("AB", script::LATIN);
            bind(&mut value, &original, &[Some(0), Some(1)], &original);
            if mutate == 0 {
                value.add('A', 0);
            } else {
                value.set_direction(Direction::RightToLeft);
            }
            assert_eq!(
                font.shape(value, &[]).shaping_status(),
                ShapingStatus::Refused(ShapingRefusal::JoiningInput)
            );
        }
    });
}

#[test]
fn exact_scalar_admission_rejects_foreign_order_count_and_positive_safety() {
    with_font(|font| {
        let original = source("AA", script::LATIN);
        let mut value = input("AA", script::LATIN);
        assert_eq!(
            value.set_joining_sources(&original, &[Some(0)], &original, &[0, 1]),
            Err(JoiningInputError::Length)
        );
        assert_eq!(
            value.set_joining_sources(&original, &[Some(0), Some(0)], &original, &[0, 1]),
            Err(JoiningInputError::SourceOrder)
        );
        assert_eq!(
            value.set_joining_sources(&original, &[Some(0), Some(2)], &original, &[0, 1]),
            Err(JoiningInputError::SourceIndex)
        );
        assert_eq!(
            value.set_joining_sources(&original, &[Some(0), Some(1)], &original, &[1, 0]),
            Err(JoiningInputError::SourceOrder)
        );
        bind(&mut value, &original, &[Some(0), Some(1)], &original);
        value.set_flags(BufferFlags::PRODUCE_SAFE_TO_INSERT_TATWEEL);
        assert_eq!(
            font.shape(value, &[]).shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::JoiningProfile)
        );
    });
}

#[test]
fn real_normalization_keeps_decomposition_reordering_and_transparent_composition_custody() {
    with_font(|font| {
        for text in ["\u{212b}", "A\u{301}\u{327}", "A\u{30a}"] {
            let original = source(text, script::LATIN);
            let mut value = input(text, script::LATIN);
            let map: Vec<_> = (0..value.len()).map(Some).collect();
            bind(&mut value, &original, &map, &original);
            let actual = font.shape(value, &[]);
            assert_eq!(actual.shaping_status(), ShapingStatus::Complete, "{text:?}");
            let ordinary = font.shape(input(text, script::LATIN), &[]);
            assert_eq!(
                actual
                    .glyph_infos()
                    .iter()
                    .map(|i| i.glyph_id)
                    .collect::<Vec<_>>(),
                ordinary
                    .glyph_infos()
                    .iter()
                    .map(|i| i.glyph_id)
                    .collect::<Vec<_>>()
            );
            let positions = |values: &[GlyphPosition]| {
                values
                    .iter()
                    .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                positions(actual.glyph_positions()),
                positions(ordinary.glyph_positions())
            );
        }
        let original = source("A", script::LATIN);
        let presentation = source("A\u{30a}", script::LATIN);
        let mut value = input("A\u{30a}", script::LATIN);
        bind(&mut value, &original, &[Some(0), None], &presentation);
        assert_eq!(
            font.shape(value, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}

fn cmap(groups: &[(u32, u32)], variant: bool) -> Vec<u8> {
    let header = if variant { 20u32 } else { 12u32 };
    let length = 16 + groups.len() as u32 * 12;
    let mut bytes = Vec::new();
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((if variant { 2u16 } else { 1u16 }).to_be_bytes());
    bytes.extend(3u16.to_be_bytes());
    bytes.extend(10u16.to_be_bytes());
    bytes.extend(header.to_be_bytes());
    if variant {
        bytes.extend(0u16.to_be_bytes());
        bytes.extend(5u16.to_be_bytes());
        bytes.extend((header + length).to_be_bytes());
    }
    bytes.extend(12u16.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend(length.to_be_bytes());
    bytes.extend(0u32.to_be_bytes());
    bytes.extend((groups.len() as u32).to_be_bytes());
    for &(cp, gid) in groups {
        bytes.extend(cp.to_be_bytes());
        bytes.extend(cp.to_be_bytes());
        bytes.extend(gid.to_be_bytes());
    }
    if variant {
        bytes.extend(14u16.to_be_bytes());
        bytes.extend(30u32.to_be_bytes());
        bytes.extend(1u32.to_be_bytes());
        bytes.extend([0, 0xfe, 0x0f]);
        bytes.extend(0u32.to_be_bytes());
        bytes.extend(21u32.to_be_bytes());
        bytes.extend(1u32.to_be_bytes());
        bytes.extend([0, 0x18, 0x20]);
        bytes.extend(36u16.to_be_bytes());
    }
    bytes
}

#[test]
fn real_hangul_composition_agrees_for_original_bearings_but_refuses_mixed_ownership() {
    let cmap = cmap(&[(0x1100, 35), (0x1161, 36), (0xac00, 58)], false);
    with_tables(&[(*b"cmap", &cmap)], |font| {
        let original = source("\u{1100}\u{1161}", script::HANGUL);
        let mut value = input("\u{1100}\u{1161}", script::HANGUL);
        bind(&mut value, &original, &[Some(0), Some(1)], &original);
        let result = font.shape(value, &[]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            result
                .glyph_infos()
                .iter()
                .map(|g| g.glyph_id)
                .collect::<Vec<_>>(),
            [58]
        );
        let mut mixed = input("\u{1100}\u{1161}", script::HANGUL);
        bind(&mut mixed, &original, &[Some(0), None], &original);
        assert_eq!(
            font.shape(mixed, &[]).shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::JoiningCorrespondence)
        );
    });
}

#[test]
fn actual_uvs_contraction_preserves_transparent_scalar_provenance() {
    let cmap = cmap(&[(0x1820, 35)], true);
    with_tables(&[(*b"cmap", &cmap)], |font| {
        let original = source("\u{1820}\u{fe0f}", script::MONGOLIAN);
        let ordinary = font.shape(input("\u{1820}\u{fe0f}", script::MONGOLIAN), &[]);
        assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            ordinary
                .glyph_infos()
                .iter()
                .map(|g| g.glyph_id)
                .collect::<Vec<_>>(),
            [36]
        );
        let mut value = input("\u{1820}\u{fe0f}", script::MONGOLIAN);
        bind(&mut value, &original, &[Some(0), Some(1)], &original);
        assert_eq!(
            font.shape(value, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}

#[test]
fn aggregate_growth_and_work_guards_stop_before_lineage_allocation() {
    let original = source("AA", script::LATIN);
    let mut value = input("AA", script::LATIN);
    bind(&mut value, &original, &[Some(0), Some(1)], &original);
    value.0.max_len = 1;
    begin(&mut value.0);
    assert_eq!(
        value.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
    assert!(value.0.joining.nodes.is_empty());
    let mut value = input("AA", script::LATIN);
    bind(&mut value, &original, &[Some(0), Some(1)], &original);
    value.0.max_ops = 0;
    begin(&mut value.0);
    assert_eq!(
        value.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
    );
    assert!(value.0.joining.nodes.is_empty());
    assert_eq!(original.0.scalars.len(), 2);
}

#[test]
fn tiny_emitted_body_cannot_hide_a_retained_full_domain_from_aggregate_admission() {
    let original = source(&"A".repeat(400_000), script::LATIN);
    let presentation = source("AA", script::LATIN);
    let mut value = input("AA", script::LATIN);
    assert_eq!(
        value.set_joining_sources(&original, &[Some(0), Some(1)], &presentation, &[0, 1]),
        Err(JoiningInputError::GrowthLimit)
    );
    assert!(value.0.joining.request.is_none());
    assert!(matches!(
        JoiningSource::analyze("A", script::LATIN, Direction::TopToBottom, &[]),
        Err(JoiningSourceError::UnsupportedDirection)
    ));
}

#[test]
fn dag_relocation_refuses_a_peak_that_exceeds_allowance_even_when_retained_capacity_fits() {
    let original = source("AAAA", script::LATIN);
    let mut value = input("AAAA", script::LATIN);
    bind(
        &mut value,
        &original,
        &[Some(0), Some(1), Some(2), Some(3)],
        &original,
    );
    begin(&mut value.0);
    let old = value.0.joining.nodes.capacity();
    let current = storage_bytes(&value.0, value.0.len).unwrap() + value.0.joining.bytes;
    let retained = current + old * size_of::<Lineage>();
    let peak = current + old * 2 * size_of::<Lineage>();
    value.0.max_len = retained.div_ceil(allowance_row_bytes());
    let allowance = value.0.max_len * allowance_row_bytes();
    assert!(retained <= allowance && allowance < peak);
    assert_eq!(union(&mut value.0, 1, 2), 0);
    assert_eq!(
        value.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
    assert_eq!(value.0.joining.nodes.len(), 4);
    assert_eq!(value.0.joining.nodes.capacity(), old);
}

#[test]
fn normalization_row_relocation_accounts_binding_and_old_plus_new_capacity() {
    let original = source("AAAA", script::LATIN);
    let mut value = input("AAAA", script::LATIN);
    bind(
        &mut value,
        &original,
        &[Some(0), Some(1), Some(2), Some(3)],
        &original,
    );
    let info_capacity = value.0.info.capacity();
    let pos_capacity = value.0.pos.capacity();
    let retained = storage_bytes(&value.0, 8).unwrap() + value.0.joining.bytes;
    let overlap =
        (info_capacity * size_of::<GlyphInfo>()).max(pos_capacity * size_of::<GlyphPosition>());
    value.0.max_len = retained.div_ceil(allowance_row_bytes());
    let allowance = value.0.max_len * allowance_row_bytes();
    assert!(retained <= allowance && allowance < retained + overlap);
    assert!(!value.0.ensure(8));
    assert_eq!(
        value.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
    assert_eq!(value.0.info.capacity(), info_capacity);
    assert_eq!(value.0.pos.capacity(), pos_capacity);
}

#[test]
fn public_bound_reserve_uses_binding_allowance_before_shape_entry() {
    let original = source("AA", script::LATIN);
    let mut value = input("AA", script::LATIN);
    bind(&mut value, &original, &[Some(0), Some(1)], &original);
    let capacities = (value.0.info.capacity(), value.0.pos.capacity());
    assert!(!value.reserve(100_000));
    assert_eq!(
        value.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
    assert_eq!(
        (value.0.info.capacity(), value.0.pos.capacity()),
        capacities
    );
    assert_eq!(value.len(), 2);
}

#[test]
fn binding_admission_includes_borrowed_correspondence_overlap() {
    let original = source("AAAA", script::LATIN);
    let mut value = input("AAAA", script::LATIN);
    let allowance = hb_buffer_t::MAX_LEN_MIN * allowance_row_bytes();
    let owned = retained_bytes(&original, &original).unwrap() + 4 * size_of::<InputMapping>();
    let capacity = (allowance - owned) / row_bytes();
    assert!(value.reserve(capacity));
    let rows = storage_bytes(&value.0, value.len()).unwrap();
    let borrowed = 4 * (size_of::<Option<usize>>() + size_of::<usize>());
    assert!(rows + owned <= allowance && allowance < rows + owned + borrowed);
    assert_eq!(
        value.set_joining_sources(
            &original,
            &[Some(0), Some(1), Some(2), Some(3)],
            &original,
            &[0, 1, 2, 3]
        ),
        Err(JoiningInputError::GrowthLimit)
    );
    assert!(value.0.joining.request.is_none());
}

#[test]
fn arabic_profile_normalization_refusal_releases_phase_variables_before_clear_reuse() {
    // The public provider accepts an explicit Arabic plan for Unicode input.
    // Canonical U+09CB decomposes into two spacing marks (both exact class U),
    // and this real face has only the two decomposition glyphs. One original
    // bearing therefore has two normalized bearings: precise refusal is required.
    let cmap = cmap(&[(0x09be, 35), (0x09c7, 36)], false);
    with_tables(&[(*b"cmap", &cmap)], |font| {
        let original = source("\u{09cb}", script::ARABIC);
        let control = font.shape(input("\u{09cb}", script::ARABIC), &[]);
        assert_eq!(control.shaping_status(), ShapingStatus::Complete);
        assert_eq!(control.glyph_infos().len(), 2);
        let mut value = input("\u{09cb}", script::ARABIC);
        bind(&mut value, &original, &[Some(0)], &original);
        let refused = font.shape(value, &[]);
        assert_eq!(
            refused.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::JoiningCorrespondence)
        );
        let mut reused = refused.clear();
        reused.push_str("\u{09cb}");
        reused.set_script(script::ARABIC);
        reused.set_direction(Direction::LeftToRight);
        let complete = font.shape(reused, &[]);
        assert_eq!(complete.shaping_status(), ShapingStatus::Complete);
        assert_eq!(complete.glyph_infos().len(), 2);
    });
}

#[test]
fn mongolian_fvs_retains_exact_identity_and_its_original_postprocessed_action() {
    let original = source("\u{1820}\u{180b}", script::MONGOLIAN);
    assert_ne!(
        original.0.scalars[1].action,
        ot_shaper_arabic::no_joining_action()
    );
    assert_eq!(original.0.scalars[0].action, original.0.scalars[1].action);
    with_font(|font| {
        let mut value = input("\u{1820}\u{180b}", script::MONGOLIAN);
        bind(&mut value, &original, &[Some(0), Some(1)], &original);
        assert_eq!(
            font.shape(value, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}
