//! Real raw chooser uses the same continuation/shaping/placement owners.
use super::*;
use crate::{
    HyphenateCharacter, HyphenationPlacement, Hyphens, LineHeight, NativeHyphenationConvention,
    Range, Text,
};
#[test]
fn raw_after_carries_one_real_prefix_and_preserves_raw_metric_profiles() {
    let source = Source::new("AB\u{ad}CDE");
    for quantize in [false, true] {
        let mut system = system();
        for (placement, advances) in [
            (HyphenationPlacement::LogicalBefore, [30.0, 30.0]),
            (HyphenationPlacement::LogicalAfter, [20.0, 40.0]),
        ] {
            let style = Style {
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                locale: Some("en-Latn-x-native".into()),
                hyphens: Hyphens::Manual,
                hyphenate_character: HyphenateCharacter::String(Text::from("A")),
                native_hyphenation_convention: Some(
                    NativeHyphenationConvention::language_script(
                        "en-Latn-x-native",
                        "Latn",
                        Text::from("A"),
                        placement,
                    )
                    .unwrap(),
                ),
                ..style()
            };
            let unbroken = system
                .layout(
                    source.clone(),
                    style.clone(),
                    crate::Options {
                        quantize,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                unbroken.source().text().code_units(),
                [65, 66, 173, 67, 68, 69]
            );
            assert_eq!(unbroken.lines().len(), 1);
            let layout = system
                .layout(
                    source.clone(),
                    style,
                    crate::Options {
                        width: Some(40.0),
                        quantize,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                layout.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
                [Range::new(0, 3), Range::new(3, 6)]
            );
            let mut actual = vec![0.0; layout.lines().len()];
            for run in layout.inline_metric_facts().runs() {
                actual[run.line()] += run.advance();
            }
            assert_eq!(actual, advances);
            let mut glyphs = vec![Vec::new(); 2];
            for run in layout.glyph_runs() {
                for glyph in run.glyphs() {
                    if glyph.advance() != 0.0 {
                        glyphs[run.line()].push((glyph.id(), glyph.range(), glyph.advance()));
                    }
                }
            }
            let a = (35, Range::new(0, 1), 10.0);
            let b = (36, Range::new(1, 2), 10.0);
            let c = (37, Range::new(3, 4), 10.0);
            let d = (38, Range::new(4, 5), 10.0);
            let e = (39, Range::new(5, 6), 10.0);
            let indication = (35, Range::new(3, 3), 10.0);
            assert_eq!(
                glyphs,
                if placement == HyphenationPlacement::LogicalAfter {
                    vec![vec![a, b], vec![indication, c, d, e]]
                } else {
                    vec![vec![a, b, indication], vec![c, d, e]]
                }
            );
            for run in layout.run_views() {
                assert_eq!(
                    run.face().bytes(),
                    include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf")
                );
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            }
        }
    }
}
