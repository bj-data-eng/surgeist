//! Supplemental owning-boundary qualification; the published public RED is separate.
use super::{
    tests::{graphemes, ordinary_boundaries},
    *,
};
use crate::WordBreak;

fn cuts(text: &str, word: WordBreak, strictness: LineBreak) -> Vec<usize> {
    ordinary_boundaries(text, word, strictness)
        .into_iter()
        .enumerate()
        .filter_map(|(atom, boundary)| (boundary == Boundary::Line).then_some(atom))
        .collect()
}

#[test]
fn all_letter_and_number_units_use_the_css_rule_beyond_provider_classes() {
    let classes = CodePointMapData::<Class>::new();
    let categories = CodePointMapData::<GeneralCategory>::new();
    assert_eq!(classes.get32(0x5d0), Class::HebrewLetter);
    assert_eq!(classes.get32(0x1b05), Class::Aksara);
    assert_eq!(categories.get32(0x1b05), GeneralCategory::OtherLetter);
    assert_eq!(categories.get32(0x2160), GeneralCategory::LetterNumber);
    for text in ["אב", "\u{1b05}\u{1b05}", "\u{2160}\u{2161}", "A1"] {
        for strictness in [LineBreak::Strict, LineBreak::Normal, LineBreak::Loose] {
            assert_eq!(cuts(text, WordBreak::BreakAll, strictness), [1], "{text:?}");
        }
    }
    for word in [WordBreak::Normal, WordBreak::KeepAll, WordBreak::BreakWord] {
        assert!(cuts("אב", word, LineBreak::Strict).is_empty());
    }
}

#[test]
fn complete_egcs_and_native_khmer_units_keep_their_indivisible_edges() {
    for (text, expected) in [
        ("א\u{5b0}ב", vec![2]),
        ("\u{1100}\u{1161}\u{1100}\u{1161}", vec![2]),
        ("\u{1780}\u{17d2}\u{1781}\u{1780}", vec![3]),
    ] {
        assert_eq!(
            cuts(text, WordBreak::BreakAll, LineBreak::Strict),
            expected,
            "{text:?}"
        );
    }
}

#[test]
fn break_all_projection_maps_supplementary_representatives_to_exact_original_bytes() {
    assert_eq!(
        CodePointMapData::<Class>::new().get32(0x20000),
        Class::Ideographic
    );
    let text = "\u{10400}אב\u{301}א";
    let projection = SourceProjection::new(&crate::Source::new(text)).unwrap();
    let egcs = graphemes(&projection);
    let units = super::super::line_units::LineBreakUnits::new(&projection, None).unwrap();
    let css = BreakAll::new(
        &projection,
        &egcs,
        &units,
        &(0..projection.atoms.len()),
        &projection.segmentation,
    );
    let input = css.input.as_ref().unwrap();
    let (actual, _) = provider(
        input,
        Policy {
            word: OrdinaryWordBreak::BreakAll,
            ..Policy::BASELINE
        },
    );
    assert_eq!(
        actual
            .iter()
            .enumerate()
            .filter_map(|(byte, &cut)| cut.then_some(byte))
            .collect::<Vec<_>>(),
        [0, 4, 6, 10, 12]
    );
    assert_eq!(projection.segmentation, text);
    assert_eq!(
        cuts(text, WordBreak::BreakAll, LineBreak::Strict),
        [1, 2, 4]
    );
}

#[test]
fn break_all_required_dash_rules_use_the_same_effective_letter_class() {
    for dash in ['\u{2010}', '\u{2013}'] {
        let text = format!("א{dash}א");
        assert!(cuts(&text, WordBreak::BreakAll, LineBreak::Loose).contains(&1));
        for strictness in [LineBreak::Strict, LineBreak::Normal] {
            assert!(!cuts(&text, WordBreak::BreakAll, strictness).contains(&1));
        }
    }
}

#[test]
fn raw_punctuation_and_glue_keep_their_context_around_mapped_letters() {
    for text in ["(אב)", "“אב”", "「אב」"] {
        for strictness in [LineBreak::Strict, LineBreak::Normal, LineBreak::Loose] {
            assert_eq!(cuts(text, WordBreak::BreakAll, strictness), [2], "{text:?}");
        }
    }
    for text in ["א\u{a0}ב", "א\u{2060}ב", "א\u{200d}ב"] {
        assert!(cuts(text, WordBreak::BreakAll, LineBreak::Strict).is_empty());
    }
    for text in ["א ב", "א\u{200b}ב"] {
        assert_eq!(cuts(text, WordBreak::BreakAll, LineBreak::Strict), [2]);
    }
}

#[test]
fn only_actual_letter_units_are_promoted_and_already_supported_input_stays_borrowed() {
    for (text, expected) in [
        ("\u{301}אב", vec![false, true, true]),
        ("\u{202f}\u{363}א", vec![false, false, true]),
    ] {
        let projection = SourceProjection::new(&crate::Source::new(text)).unwrap();
        let egcs = graphemes(&projection);
        let units = super::super::line_units::LineBreakUnits::new(&projection, None).unwrap();
        let css = BreakAll::new(
            &projection,
            &egcs,
            &units,
            &(0..projection.atoms.len()),
            &projection.segmentation,
        );
        assert_eq!(css.ideographic, expected);
    }
    let text = "abc123";
    let projection = SourceProjection::new(&crate::Source::new(text)).unwrap();
    let egcs = graphemes(&projection);
    let units = super::super::line_units::LineBreakUnits::new(&projection, None).unwrap();
    let css = BreakAll::new(
        &projection,
        &egcs,
        &units,
        &(0..projection.atoms.len()),
        &projection.segmentation,
    );
    assert!(css.input.is_none());
    let input = ProviderInput::without_lexical_models(text);
    assert!(std::ptr::eq(input.text().as_ptr(), text.as_ptr()));
    assert_eq!(
        cuts(text, WordBreak::BreakAll, LineBreak::Strict),
        [1, 2, 3, 4, 5]
    );
}
