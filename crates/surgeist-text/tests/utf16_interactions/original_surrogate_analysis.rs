//! Original-source bidi analysis must precede glyph-buffer replacement.
use super::*;

const SOURCE_ID: Id = Id::from_u64(1080);
const SOURCE_REVISION: u64 = 7;

fn source(first: u16) -> Source {
    Source::identified(
        Text::from_code_units(vec![first, 0x05d0]),
        SOURCE_ID,
        SOURCE_REVISION,
    )
}

fn assert_original_source_and_borrowed_runs(output: &Layout, units: [u16; 2], revision: u64) {
    assert_eq!(output.source().text().code_units(), units);
    assert_eq!(output.source().id(), Some(SOURCE_ID));
    assert_eq!(output.source().revision(), revision);
    assert_eq!(output.key().source().id(), Some(SOURCE_ID));
    assert_eq!(output.key().source().revision().get(), revision);
    assert_eq!(output.lines().len(), 1);
    assert_eq!(output.lines()[0].range(), Range::new(0, 2));

    let mut covered = [false; 2];
    for run in output.run_views() {
        let range = run.range();
        assert!(range.start < range.end && range.end <= units.len());
        // This loan is read through the actual immutable Layout's source;
        // provider UTF-8 byte offsets cannot substitute for these UTF-16 cuts.
        let original = &output.source().text().code_units()[range.start..range.end];
        assert_eq!(original, &units[range.start..range.end]);
        covered[range.start..range.end].fill(true);
        assert_eq!(run.face().bytes(), AHEM);
    }
    assert_eq!(covered, [true, true], "real runs cover both original units");
}

#[test]
fn literal_replacement_and_broken_bar_keep_distinct_original_sources_and_rtl_base() {
    let mut system = registered_ahem();
    let replacement = system.layout(source(0xfffd), style(), options()).unwrap();
    let broken_bar = system.layout(source(0x00a6), style(), options()).unwrap();
    // Pinned DerivedBidiClass18 gives both literals ON and U05D0 R.
    // UAX9rev52 P2/P3 therefore selects RTL from the Aleph, not the neutral.
    assert_eq!(replacement.direction(), Direction::RightToLeft);
    assert_eq!(broken_bar.direction(), Direction::RightToLeft);
    assert_ne!(replacement.key(), broken_bar.key());
    drop(system);
    assert_original_source_and_borrowed_runs(&replacement, [0xfffd, 0x05d0], SOURCE_REVISION);
    assert_original_source_and_borrowed_runs(&broken_bar, [0x00a6, 0x05d0], SOURCE_REVISION);
}

fn assert_original_isolated_surrogate_controls_base_direction(unit: u16) {
    let mut system = registered_ahem();
    let original = source(unit);
    let literal = system.layout(source(0xfffd), style(), options()).unwrap();
    let broken_bar = system.layout(source(0x00a6), style(), options()).unwrap();
    assert_eq!(literal.direction(), Direction::RightToLeft);
    assert_eq!(broken_bar.direction(), Direction::RightToLeft);
    assert_original_source_and_borrowed_runs(&literal, [0xfffd, 0x05d0], SOURCE_REVISION);

    // docs/text-layout-contract.md selects original property analysis before
    // glyph-only replacement. The pinned DerivedBidiClass18 general @missing
    // rule makes an isolated D800 or DC00 L; neither has a range override.
    // UAX9rev52 P2/P3 therefore selects LTR before the later Hebrew R.
    let output = system.layout(original.clone(), style(), options()).expect(
        "original isolated units must reach original-source analysis before glyph replacement",
    );
    assert_eq!(output.direction(), Direction::LeftToRight);
    assert_ne!(output.key(), literal.key());
    assert_ne!(output.key(), broken_bar.key());
    assert_original_source_and_borrowed_runs(&output, [unit, 0x05d0], SOURCE_REVISION);

    let edited = TextEdit::replace(
        &original,
        Range::new(0, 1),
        Text::from_code_units(vec![0xfffd]),
    )
    .unwrap()
    .apply_to(original.clone())
    .unwrap();
    let replacement = system.layout(edited, style(), options()).unwrap();
    assert_eq!(replacement.direction(), Direction::RightToLeft);
    assert_ne!(output.key(), replacement.key());
    drop(original);
    drop(system);

    // A later source revision and the producer's destruction cannot rewrite
    // either retained layout or its source-associated borrowed run views.
    assert_eq!(output.direction(), Direction::LeftToRight);
    assert_original_source_and_borrowed_runs(&output, [unit, 0x05d0], SOURCE_REVISION);
    assert_original_source_and_borrowed_runs(&replacement, [0xfffd, 0x05d0], SOURCE_REVISION + 1);
}

#[test]
fn original_high_surrogate_before_hebrew_is_ltr_unlike_literal_replacement() {
    assert_original_isolated_surrogate_controls_base_direction(0xd800);
}

#[test]
fn original_low_surrogate_before_hebrew_is_ltr_unlike_literal_replacement() {
    assert_original_isolated_surrogate_controls_base_direction(0xdc00);
}
