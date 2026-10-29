//! Existing-callable RED for supplied backward vertical positioning and native
//! mirroring fallback. Expected glyphs and packets come from authored SFNT
//! operands, not a second implementation of mirroring or observed output.
use super::{font, old_positions, positions, qualify};
use crate::hb::completion_fixture::words;
use crate::{
    script, ComponentCoverage, Direction, GlyphBuffer, ShapePlan, Shaper, ShapingStatus,
    UnicodeBuffer, VerticalOriginReference,
};
use alloc::{vec, vec::Vec};
use core::convert::Infallible;
use read_fonts::TableProvider;

const LEFT: char = '\u{3008}';
const RIGHT: char = '\u{3009}';
const CLUSTERS: [u32; 2] = [17, 901];

fn word(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn cmap(entries: &[(u32, u16)]) -> Vec<u8> {
    let mut entries = entries.to_vec();
    entries.sort_unstable();
    assert!(entries.windows(2).all(|pair| pair[0].0 != pair[1].0));
    let mut result = words(&[0, 1, 3, 10]);
    result.extend(12_u32.to_be_bytes());
    result.extend(words(&[12, 0]));
    result.extend(
        u32::try_from(16 + 12 * entries.len())
            .unwrap()
            .to_be_bytes(),
    );
    result.extend(0_u32.to_be_bytes());
    result.extend(u32::try_from(entries.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in entries {
        result.extend(scalar.to_be_bytes());
        result.extend(scalar.to_be_bytes());
        result.extend(u32::from(glyph).to_be_bytes());
    }
    result
}

fn mappings(mirrored: bool) -> Vec<(u32, u16)> {
    let mut entries = vec![
        (32, font::SPACE),
        (46, font::DOT),
        (65, font::A),
        (66, font::B),
        (82, font::R),
        (u32::from(LEFT), font::A),
    ];
    if mirrored {
        entries.push((u32::from(RIGHT), font::B));
    }
    entries
}

fn with_mirror_font(mirrored: bool, f: impl FnOnce(&Shaper<'_>)) {
    let cmap = cmap(&mappings(mirrored));
    let gsub = font::layout_table(b"rtlm", &[font::replace()]);
    // Independently read the actual lookup's coverage and replacement operand.
    let list = usize::from(word(&gsub, 8));
    assert_eq!(word(&gsub, list), 1);
    let lookup = list + usize::from(word(&gsub, list + 2));
    assert_eq!((word(&gsub, lookup), word(&gsub, lookup + 4)), (1, 1));
    let sub = lookup + usize::from(word(&gsub, lookup + 6));
    assert_eq!(
        (word(&gsub, sub), word(&gsub, sub + 4), word(&gsub, sub + 6)),
        (2, 1, 58)
    );
    let coverage = sub + usize::from(word(&gsub, sub + 2));
    assert_eq!(
        (
            word(&gsub, coverage),
            word(&gsub, coverage + 2),
            word(&gsub, coverage + 4)
        ),
        (1, 1, 35)
    );
    font::with_cmap_font(true, true, Some(&gsub), None, &[], &cmap, |font| {
        qualify(font, true, true);
        assert_eq!(
            font.get_nominal_glyph(u32::from(LEFT)).unwrap().to_u32(),
            35
        );
        assert_eq!(
            font.get_nominal_glyph(u32::from(RIGHT)).map(|g| g.to_u32()),
            mirrored.then_some(36)
        );
        // Unicode17 BidiMirroring.txt supplies U3008 <-> U3009; both are
        // VerticalOrientation=Tr. The fixture maps neither vertical form, so
        // an unrelated vertical presentation fallback cannot select a glyph.
        assert!(font.get_nominal_glyph(0xfe3f).is_none());
        assert!(font.get_nominal_glyph(0xfe40).is_none());
        let gsub = font.font.gsub().unwrap();
        let features = gsub.feature_list().unwrap();
        assert_eq!(features.feature_count(), 1);
        assert_eq!(
            features.feature_records()[0].feature_tag().into_bytes(),
            *b"rtlm"
        );
        let list = gsub.lookup_list().unwrap();
        assert_eq!(list.lookup_count(), 1);
        assert!(matches!(
            list.lookups().get(0).unwrap(),
            read_fonts::tables::gsub::SubstitutionLookup::Single(_)
        ));
        for (glyph, expected) in [
            (35, (1200, 1001, 900)),
            (36, (1400, 1000, 1000)),
            (58, (1600, 1201, 1000)),
            (16, (701, 500, 900)),
        ] {
            let metrics = font::metrics(font, glyph);
            assert_eq!(
                (
                    metrics.advance(),
                    metrics.origin_x_twice(),
                    metrics.origin_y()
                ),
                expected
            );
        }
        f(font);
    });
}

fn input(direction: Direction) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    input.observe_components();
    input.add(LEFT, CLUSTERS[0]);
    input.add('.', CLUSTERS[1]);
    input.set_direction(direction);
    input.set_script(script::LATIN);
    input
}

fn ordinary(font: &Shaper<'_>, direction: Direction) -> GlyphBuffer {
    font.shape(input(direction), &[])
}

fn ids(buffer: &GlyphBuffer) -> Vec<(u32, u32)> {
    buffer
        .glyph_infos()
        .iter()
        .map(|g| (g.glyph_id, g.cluster))
        .collect()
}

#[test]
fn mirrored_cmap_backward_supplied_metrics_use_actual_glyph_and_original_input_ordinals() {
    with_mirror_font(true, |font| {
        let ttb = ordinary(font, Direction::TopToBottom);
        assert_eq!(ids(&ttb), [(35, 17), (16, 901)]);
        assert_eq!(
            old_positions(&ttb),
            [(0, -1200, -500, -900), (0, -701, -250, -900)]
        );
        let btt = ordinary(font, Direction::BottomToTop);
        assert_eq!(ids(&btt), [(16, 901), (36, 17)]);
        assert_eq!(
            old_positions(&btt),
            [(0, -701, -250, -900), (0, -1400, -500, -1000)]
        );
        let rtl = ordinary(font, Direction::RightToLeft);
        assert_eq!(ids(&rtl), [(16, 901), (36, 17)]);
        assert_eq!(old_positions(&rtl), [(500, 0, 0, 0), (1000, 0, 0, 0)]);
        let source = input(Direction::BottomToTop);
        let plan = ShapePlan::new(font, Direction::BottomToTop, Some(script::LATIN), None, &[]);
        let mut queried = Vec::new();
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |glyph| {
                    queried.push(glyph);
                    Ok::<_, Infallible>(font::metrics(font, glyph))
                },
            )
            .expect("BTT must use the existing native mirroring and supplied-metric operation");
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            result
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(16, 901), (36, 17)]
        );
        assert_eq!(
            positions(&result),
            [
                (0.0, -701.0, -250.0, -900.0),
                (0.0, -1400.0, -500.0, -1000.0)
            ]
        );
        queried.sort_unstable();
        queried.dedup();
        assert_eq!(queried, [16, 36]);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.input_len(), 2);
        assert_eq!(receipt.components(0), Some(&[1][..]));
        assert_eq!(receipt.components(1), Some(&[0][..]));
        assert!(receipt.deleted_input_positions().is_empty());
        let di = result.default_ignorable_receipt().unwrap();
        assert!(di.residual_input_positions().is_empty());
        assert!(di.deleted_input_positions().is_empty());
        assert!(di.residual_glyph_positions().is_empty());
    });
}

#[test]
fn default_backward_rtlm_fallback_is_selected_before_supplied_used_glyph_metrics() {
    with_mirror_font(false, |font| {
        let ttb = ordinary(font, Direction::TopToBottom);
        assert_eq!(ids(&ttb), [(35, 17), (16, 901)]);
        assert_eq!(
            old_positions(&ttb),
            [(0, -1200, -500, -900), (0, -701, -250, -900)]
        );
        let rtl = ordinary(font, Direction::RightToLeft);
        assert_eq!(ids(&rtl), [(16, 901), (58, 17)]);
        assert_eq!(old_positions(&rtl), [(500, 0, 0, 0), (1201, 0, 0, 0)]);
        // This assertion isolates native plan fallback independently of the
        // supplied operation's earlier BTT admission check.
        let btt = ordinary(font, Direction::BottomToTop);
        assert_eq!(
            ids(&btt),
            [(16, 901), (58, 17)],
            "an unavailable Unicode mirror must retain native rtlm fallback"
        );
        assert_eq!(
            old_positions(&btt),
            [(0, -701, -250, -900), (0, -1600, -600, -1000)]
        );
        let source = input(Direction::BottomToTop);
        let plan = ShapePlan::new(font, Direction::BottomToTop, Some(script::LATIN), None, &[]);
        let mut queried = Vec::new();
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |glyph| {
                    queried.push(glyph);
                    Ok::<_, Infallible>(font::metrics(font, glyph))
                },
            )
            .expect("BTT fallback must query the actual substituted glyph");
        assert_eq!(
            result
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(16, 901), (58, 17)]
        );
        assert_eq!(
            positions(&result),
            [
                (0.0, -701.0, -250.0, -900.0),
                (0.0, -1600.0, -600.5, -1000.0)
            ]
        );
        queried.sort_unstable();
        queried.dedup();
        assert_eq!(queried, [16, 58]);
    });
}
