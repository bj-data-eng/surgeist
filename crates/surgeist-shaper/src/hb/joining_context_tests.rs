//! Functional qualification of a new invocation-local mechanism observation.
//! No executable preimplementation predicate existed for this API.
use crate::hb::completion_fixture::{with_tables, words};
use crate::{
    script, ComponentCoverage, ComponentPath, Direction, Feature, GlyphBuffer, JoiningContextUse,
    JoiningSource, Script, Shaper, ShapingRefusal, ShapingStatus, Tag, UnicodeBuffer,
};
use alloc::{vec, vec::Vec};
use read_fonts::{types::GlyphId, TableProvider};

fn cmap() -> Vec<u8> {
    let scalars = [0x41u32, 0x628, 0x712, 0x7ca, 0x11005];
    let mut result = words(&[0, 1, 3, 10]);
    result.extend(12u32.to_be_bytes());
    result.extend(words(&[12, 0]));
    for value in [16 + 12 * scalars.len() as u32, 0, scalars.len() as u32] {
        result.extend(value.to_be_bytes());
    }
    for scalar in scalars {
        for value in [scalar, scalar, 35] {
            result.extend(value.to_be_bytes());
        }
    }
    result
}
fn gsub(script: &[u8; 4], forms: bool, isolated: u16) -> Vec<u8> {
    let features: Vec<_> = if forms {
        vec![
            (*b"fina", 82),
            (*b"init", 100),
            (*b"isol", isolated),
            (*b"medi", 277),
        ]
    } else {
        vec![(*b"liga", 35)]
    };
    let count = u16::try_from(features.len()).unwrap();
    let mut scripts = words(&[1]);
    scripts.extend(script);
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, count]));
    scripts.extend(words(&(0..count).collect::<Vec<_>>()));
    let mut feature_list = words(&[count]);
    for (index, (tag, _)) in features.iter().enumerate() {
        feature_list.extend(tag);
        feature_list.extend(words(&[2 + 6 * count + 6 * u16::try_from(index).unwrap()]));
    }
    for index in 0..count {
        feature_list.extend(words(&[0, 1, index]));
    }
    let mut lookups = words(&[count]);
    for index in 0..count {
        lookups.extend(words(&[2 + 2 * count + 22 * index]));
    }
    for (_, replacement) in features {
        // One real format2 single substitution: nominal glyph35 -> chosen form.
        lookups.extend(words(&[1, 0, 1, 8, 2, 8, 1, replacement, 1, 1, 35]));
    }
    let mut result = words(&[
        1,
        0,
        10,
        u16::try_from(10 + scripts.len()).unwrap(),
        u16::try_from(10 + scripts.len() + feature_list.len()).unwrap(),
    ]);
    result.extend(scripts);
    result.extend(feature_list);
    result.extend(lookups);
    result
}
fn with_font(tag: &[u8; 4], forms: bool, isolated: u16, f: impl FnOnce(&Shaper<'_>)) {
    let cmap = cmap();
    let gsub = gsub(tag, forms, isolated);
    with_tables(&[(*b"cmap", &cmap), (*b"GSUB", &gsub)], |font| {
        assert_eq!(font.units_per_em(), 1000);
        for scalar in [0x41, 0x628, 0x712, 0x7ca, 0x11005] {
            assert_eq!(font.get_nominal_glyph(scalar).unwrap().to_u32(), 35);
        }
        let metrics = font.font.hmtx().unwrap();
        for glyph in [35, 36, 82, 100, 277] {
            assert_eq!(metrics.advance(GlyphId::new(glyph)), Some(1000));
        }
        f(font);
    });
}
fn input(ch: char, script: Script, direction: Direction, pre: &str, post: &str) -> UnicodeBuffer {
    let mut value = UnicodeBuffer::new();
    value.observe_components();
    value.add(ch, 0);
    value.set_script(script);
    value.set_direction(direction);
    value.set_pre_context(pre);
    value.set_post_context(post);
    value
}
fn complete(result: &GlyphBuffer, glyph: u32, expected: JoiningContextUse) {
    assert_eq!(result.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        result
            .glyph_infos()
            .iter()
            .map(|g| g.glyph_id)
            .collect::<Vec<_>>(),
        [glyph]
    );
    assert_eq!(result.glyph_positions()[0].x_advance, 1000);
    let receipt = result.component_receipt().unwrap().unwrap();
    assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
    assert_eq!(receipt.input_len(), 1);
    assert_eq!(receipt.components(0), Some(&[0][..]));
    for _ in 0..3 {
        assert_eq!(receipt.joining_context_use(), expected);
        assert_eq!(receipt.components(0), Some(&[0][..]));
    }
    assert_eq!(result.shaping_status(), ShapingStatus::Complete);
}

#[test]
fn joining_context_actual_arabic_syriac_and_joining_use_masks_select_contextual_forms() {
    for (tag, script, ch) in [
        (*b"arab", script::ARABIC, '\u{628}'),
        (*b"syrc", script::SYRIAC, '\u{712}'),
        (*b"nko ", script::NKO, '\u{7ca}'),
    ] {
        with_font(&tag, true, 35, |font| {
            let context = alloc::format!("{ch}");
            for (pre, post, glyph) in [
                ("", "", 35),
                (context.as_str(), "", 82),
                ("", context.as_str(), 100),
                (context.as_str(), context.as_str(), 277),
            ] {
                let result = font.shape(input(ch, script, Direction::RightToLeft, pre, post), &[]);
                complete(&result, glyph, JoiningContextUse::ContextualActions);
            }
        });
    }
}

#[test]
fn joining_context_default_nonjoining_and_body_only_use_do_not_claim_unicode_context_masks() {
    with_font(b"arab", true, 35, |font| {
        let value = font.shape(
            input(
                'A',
                script::ARABIC,
                Direction::RightToLeft,
                "\u{628}",
                "\u{628}",
            ),
            &[],
        );
        complete(&value, 35, JoiningContextUse::None);
        let value = font.shape(
            input(
                'A',
                script::LATIN,
                Direction::LeftToRight,
                "\u{628}",
                "\u{628}",
            ),
            &[],
        );
        complete(&value, 35, JoiningContextUse::None);
    });
    // Brahmi uses USE's body-syllable topographical feature path. A genuine
    // isolated substitution to B proves that path ran, without Unicode joining
    // context being consumed by its distinct Arabic-plan branch.
    with_font(b"brah", true, 36, |font| {
        let value = font.shape(
            input(
                '\u{11005}',
                script::BRAHMI,
                Direction::LeftToRight,
                "\u{7ca}",
                "\u{7ca}",
            ),
            &[],
        );
        complete(&value, 36, JoiningContextUse::None);
    });
}

#[test]
fn joining_context_disabled_or_absent_action_masks_remain_negative_mechanism_observations() {
    with_font(b"arab", true, 35, |font| {
        let features =
            [b"fina", b"init", b"isol", b"medi"].map(|tag| Feature::new(Tag::new(tag), 0, ..));
        let value = font.shape(
            input(
                '\u{628}',
                script::ARABIC,
                Direction::RightToLeft,
                "\u{628}",
                "\u{628}",
            ),
            &features,
        );
        complete(&value, 35, JoiningContextUse::None);
    });
    with_font(b"syrc", false, 35, |font| {
        let value = font.shape(
            input(
                '\u{712}',
                script::SYRIAC,
                Direction::RightToLeft,
                "\u{712}",
                "\u{712}",
            ),
            &[],
        );
        complete(&value, 35, JoiningContextUse::None);
    });
}

#[test]
fn joining_context_bound_source_actions_do_not_invent_external_body_components() {
    with_font(b"arab", true, 35, |font| {
        let source = JoiningSource::analyze(
            "\u{628}\u{628}",
            script::ARABIC,
            Direction::RightToLeft,
            &[],
        )
        .unwrap();
        let mut value = input('\u{628}', script::ARABIC, Direction::RightToLeft, "", "");
        value
            .set_joining_sources(&source, &[Some(0)], &source, &[0])
            .unwrap();
        let result = font.shape(value, &[]);
        complete(&result, 100, JoiningContextUse::ContextualActions);
    });
}

#[test]
fn joining_context_empty_refused_clear_and_supplied_vertical_preserve_lifecycle() {
    with_font(b"arab", true, 35, |font| {
        let mut empty = UnicodeBuffer::new();
        empty.observe_components();
        empty.set_script(script::ARABIC);
        empty.set_direction(Direction::RightToLeft);
        let result = font.shape(empty, &[]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            result
                .component_receipt()
                .unwrap()
                .unwrap()
                .joining_context_use(),
            JoiningContextUse::None
        );
        let context = "\u{64b}".repeat(6);
        let failed = font.shape(
            input(
                '\u{628}',
                script::ARABIC,
                Direction::RightToLeft,
                &context,
                "",
            ),
            &[],
        );
        assert_eq!(
            failed.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::ContextLimit)
        );
        assert!(matches!(
            failed.component_receipt(),
            Err(ShapingRefusal::ContextLimit)
        ));
        let used = font.shape(
            input(
                '\u{628}',
                script::ARABIC,
                Direction::RightToLeft,
                "",
                "\u{628}",
            ),
            &[],
        );
        complete(&used, 100, JoiningContextUse::ContextualActions);
        let mut reset = used.clear();
        reset.observe_components();
        reset.add('A', 0);
        reset.set_script(script::LATIN);
        reset.set_direction(Direction::LeftToRight);
        complete(&font.shape(reset, &[]), 35, JoiningContextUse::None);
        let value = input(
            '\u{628}',
            script::ARABIC,
            Direction::TopToBottom,
            "\u{628}",
            "\u{628}",
        );
        let plan = crate::ShapePlan::new(
            font,
            Direction::TopToBottom,
            Some(script::ARABIC),
            None,
            &[],
        );
        let vertical = font
            .shape_vertical_with_metrics(
                &plan,
                value,
                &[],
                crate::VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    Ok::<_, core::convert::Infallible>(crate::VerticalNominalMetrics::new(
                        1000, 0, 0,
                    ))
                },
            )
            .unwrap();
        assert_eq!(vertical.glyph_infos()[0].glyph_id, 35);
        assert_eq!(
            vertical
                .component_receipt()
                .unwrap()
                .unwrap()
                .joining_context_use(),
            JoiningContextUse::None
        );
    });
}

#[test]
fn joining_context_actual_aat_path_is_unqualified_instead_of_negative_evidence() {
    let morx = super::font::morx();
    super::font::with_font(true, false, None, None, &[(*b"morx", &morx)], |font| {
        let value = font.shape(
            input(
                'A',
                script::ARABIC,
                Direction::RightToLeft,
                "\u{628}",
                "\u{628}",
            ),
            &[],
        );
        assert_eq!(value.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            value
                .component_receipt()
                .unwrap()
                .unwrap()
                .joining_context_use(),
            JoiningContextUse::Unqualified(ComponentPath::AatSubstitution)
        );
    });
}
