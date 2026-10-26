//! Real raw chooser coverage; expected advances are Ahem's retained full em.
use crate::{
    Font, FontCatalogueMode, HangingPunctuation, HangingPunctuationEnd, HyphenateCharacter,
    HyphenationPlacement, Hyphens, NativeHyphenationConvention, Options, Range, Source, Style,
    System, SystemOptions, Text,
};

fn system() -> (System, Style) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        ..Style::default()
    };
    (system, style)
}

#[test]
fn raw_end_permissions_use_fit_and_keep_the_actual_punctuation() {
    for quantize in [false, true] {
        for (permission, width, fit, aligned, maximum) in [
            (None, 25.0, 20.0, 20.0, 20.0),
            (
                Some(HangingPunctuationEnd::ForceEnd),
                15.0,
                10.0,
                10.0,
                10.0,
            ),
            (
                Some(HangingPunctuationEnd::AllowEnd),
                25.0,
                10.0,
                20.0,
                20.0,
            ),
            (
                Some(HangingPunctuationEnd::AllowEnd),
                15.0,
                10.0,
                15.0,
                20.0,
            ),
        ] {
            let (mut system, mut style) = system();
            style.hanging_punctuation = HangingPunctuation::new(false, permission, false);
            let output = system
                .layout(
                    Source::new("A."),
                    style,
                    Options {
                        width: Some(width),
                        quantize,
                        ..Options::default()
                    },
                )
                .unwrap();
            assert_eq!(output.lines().len(), 1);
            assert_eq!(output.source().text().code_units(), [65, 46]);
            let line = &output.inner.lines[0].finalized;
            let measures = line.data.measurements.as_ref().unwrap();
            assert_eq!(line.data.metrics.advance, 20.0);
            assert_eq!(measures.fit().advance(), fit);
            assert_eq!(measures.alignment().advance(), aligned);
            assert_eq!(measures.max_content_contribution().advance(), maximum);
            assert_eq!(line.data.glyphs.len(), 2);
        }
    }
}

#[test]
fn raw_first_counts_an_emitted_empty_forced_line() {
    for quantize in [false, true] {
        let (mut system, mut style) = system();
        style.hanging_punctuation = HangingPunctuation::new(true, None, false);
        let first = system
            .layout(
                Source::new("(A"),
                style.clone(),
                Options {
                    width: Some(30.0),
                    quantize,
                    ..Options::default()
                },
            )
            .unwrap();
        let line = &first.inner.lines[0].finalized;
        assert_eq!(
            line.data.measurements.as_ref().unwrap().fit().advance(),
            10.0
        );
        assert_eq!(line.data.clusters[0].x, -10.0);
        assert_eq!(line.data.clusters[1].x, 0.0);
        let after_empty = system
            .layout(
                Source::new("\n(A"),
                style,
                Options {
                    width: Some(30.0),
                    quantize,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(after_empty.lines().len(), 2);
        let line = &after_empty.inner.lines[1].finalized;
        assert_eq!(
            line.selected.request.context.formatted_line.first(),
            Some(false)
        );
        assert_eq!(
            line.data.measurements.as_ref().unwrap().fit().advance(),
            20.0
        );
    }
}

#[test]
fn raw_last_keeps_the_genuine_trailing_forced_empty() {
    for quantize in [false, true] {
        let (mut system, mut style) = system();
        style.hanging_punctuation = HangingPunctuation::new(false, None, true);
        let terminal = system
            .layout(
                Source::new("A)"),
                style.clone(),
                Options {
                    width: Some(30.0),
                    quantize,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            terminal.inner.lines[0]
                .finalized
                .data
                .measurements
                .as_ref()
                .unwrap()
                .fit()
                .advance(),
            10.0
        );
        let forced = system
            .layout(
                Source::new("A)\n"),
                style,
                Options {
                    width: Some(30.0),
                    quantize,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            forced
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(3, 3)]
        );
        assert_eq!(
            forced.inner.lines[0]
                .finalized
                .data
                .measurements
                .as_ref()
                .unwrap()
                .fit()
                .advance(),
            20.0
        );
        assert_eq!(
            forced.inner.lines[1]
                .finalized
                .selected
                .request
                .context
                .formatted_line
                .last(),
            Some(true)
        );
    }
}

#[test]
fn raw_unconstrained_width_uses_maximum_and_retains_actual_first() {
    for quantize in [false, true] {
        for (declaration, text, expected) in [
            (HangingPunctuation::default(), "A.", 20.0),
            (
                HangingPunctuation::new(false, Some(HangingPunctuationEnd::AllowEnd), false),
                "A.",
                20.0,
            ),
            (
                HangingPunctuation::new(false, Some(HangingPunctuationEnd::ForceEnd), false),
                "A.",
                10.0,
            ),
            (HangingPunctuation::new(true, None, false), "(A", 10.0),
        ] {
            let (mut system, mut style) = system();
            style.hanging_punctuation = declaration;
            let output = system
                .layout(
                    Source::new(text),
                    style,
                    Options {
                        quantize,
                        ..Options::default()
                    },
                )
                .unwrap();
            let line = &output.inner.lines[0].finalized;
            assert_eq!(
                line.selected.request.context.available_inline,
                Some(expected)
            );
            assert_eq!(
                line.selected.request.context.formatted_line.first(),
                Some(true)
            );
            assert_eq!(line.data.metrics.advance, 20.0);
        }
    }
}

#[test]
fn raw_automatic_priority_measures_the_whole_receiving_manual_partition() {
    for quantize in [false, true] {
        let (mut system, mut style) = system();
        style.locale = Some("en-US".to_owned());
        style.hyphens = Hyphens::Auto;
        style.hyphenate_character = HyphenateCharacter::String(Text::from("("));
        style.native_hyphenation_convention = Some(
            NativeHyphenationConvention::language_script(
                "en-US",
                "Latn",
                Text::from("("),
                HyphenationPlacement::LogicalAfter,
            )
            .unwrap(),
        );
        style.hanging_punctuation = HangingPunctuation::new(true, None, false);
        let output = system
            .layout(
                Source::new("as\u{ad}sociate"),
                style,
                Options {
                    width: Some(75.0),
                    quantize,
                    ..Options::default()
                },
            )
            .unwrap();
        // The published en-US associate exception admits cuts2/4. The SHY
        // shifts actual cut4 to source5. Its whole manual receiving partition
        // is `(sociate` =80px, not70px: the prior manual piece is a real
        // hypothetical predecessor, despite the current actual query being First.
        assert_eq!(
            output
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 5), Range::new(5, 10)]
        );
        assert!(matches!(
            output.inner.lines[0]
                .finalized
                .selected
                .request
                .alternative
                .as_ref()
                .unwrap()
                .view()
                .definition(),
            crate::PreparedAlternativeDefinition::Automatic { .. }
        ));
        assert_eq!(output.inner.lines[1].finalized.data.metrics.advance, 60.0);
    }
}
