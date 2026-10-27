//! A real terminal constraint can change an earlier Stable cut after an edit.
use super::*;

fn identified(text: &str, revision: u64) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Manual;
    style.white_space = WhiteSpace::Collapse;
    style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
    style.overflow_wrap = OverflowWrap::Anywhere;
    PreparedParagraph::plain(
        1,
        Source::identified(text, surgeist_text::Id::from_u64(149501), revision),
        style,
    )
}
fn terminal_constraint<S: LayoutScalar>() {
    let original = identified("as\u{ad}sociate", 1);
    let width = AvailableOf::definite(S::from_f64(70.0));
    let unconstrained = batch_with_style::<S>(
        &original,
        width,
        HyphenateLimitLast::None,
        TextWrapStyle::Stable,
    );
    assert_eq!(
        line_facts(&original, &unconstrained),
        [
            (Range::new(0, 3), 30.0, true),
            (Range::new(3, 10), 70.0, false)
        ]
    );
    let old = batch_with_style::<S>(
        &original,
        width,
        HyphenateLimitLast::Always,
        TextWrapStyle::Stable,
    );
    let expected_old = [
        (Range::new(0, 8), 70.0, false),
        (Range::new(8, 10), 20.0, false),
    ];
    assert_eq!(line_facts(&original, &old), expected_old);
    let old_source = old.placed_text()[0].layout().source().clone();
    let old_preparation = original.prepared.clone();

    let revised = identified("as\u{ad}sociate ZZZZZZZ", 2);
    let changed = batch_with_style::<S>(
        &revised,
        width,
        HyphenateLimitLast::Always,
        TextWrapStyle::Stable,
    );
    // UTF-16 offsets include the actual SHY and the consumed soft-end SPACE.
    // The real manual alternative is now two Soft edges before ParagraphEnd:
    // Always restricts the middle line, not the first line's indication.
    assert_eq!(
        line_facts(&revised, &changed),
        [
            (Range::new(0, 3), 30.0, true),
            (Range::new(3, 11), 70.0, false),
            (Range::new(11, 18), 70.0, false),
        ]
    );
    let mut ends = vec![None; 3];
    for entry in changed.final_inline_fragments() {
        let fragment = entry.fragment();
        let receipt = fragment.finalized_fragment().finalized_line();
        assert_eq!(receipt.prepared_text(), revised.prepared);
        ends[fragment.line_index()] = Some(receipt.selected_line().request().context.end);
    }
    assert_eq!(
        ends,
        [
            Some(surgeist_text::LineEnd::Soft),
            Some(surgeist_text::LineEnd::Soft),
            Some(surgeist_text::LineEnd::Paragraph)
        ]
    );
    assert_eq!(
        old_source.text().code_units(),
        &revised
            .prepared
            .analyzed_source()
            .source()
            .text()
            .code_units()[..10]
    );
    assert_ne!(old_preparation, revised.prepared);
    assert_eq!(old.placed_text()[0].layout().source(), &old_source);
    assert_eq!(line_facts(&original, &old), expected_old);
    assert_eq!(
        line_facts(&original, &unconstrained),
        [
            (Range::new(0, 3), 30.0, true),
            (Range::new(3, 10), 70.0, false)
        ]
    );
}
#[test]
fn stable_keeps_always_tail_constraint_instead_of_an_absolute_edit_prefix_f32() {
    terminal_constraint::<f32>();
}
#[test]
fn stable_keeps_always_tail_constraint_instead_of_an_absolute_edit_prefix_f64() {
    terminal_constraint::<f64>();
}
