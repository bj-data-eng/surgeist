//! Actual forced/empty tails and nonpositive content through public composition.
use super::*;
use surgeist_text::{
    HyphenateCharacter, HyphenationProvenance, HyphenationResource, Hyphens, LineEnd, OverflowWrap,
    PreparedAlternativeDefinition, PreparedBreakKind, Range, Text, WhiteSpace,
};

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

fn paragraph(text: &str, y_spacing: f64) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
    style.white_space = WhiteSpace::Collapse;
    style.wrap = Wrap::Word;
    style.overflow_wrap = OverflowWrap::Normal;
    let mut source = Source::new(text);
    for (offset, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            source.span(
                Range::new(offset, offset + 1),
                surgeist_text::Style {
                    white_space: WhiteSpace::Preserve,
                    ..style.clone()
                },
            );
        } else if byte == b'Y' {
            source.span(
                Range::new(offset, offset + 1),
                surgeist_text::Style {
                    letter_spacing: y_spacing,
                    ..style.clone()
                },
            );
        }
    }
    PreparedParagraph::plain(1, source, style)
}

fn check<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    text: &str,
    policy: HyphenateLimitLast,
    expected: &[(Range, f64, LineEnd)],
) {
    let p = &paragraph.prepared;
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    let word = p
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    assert_eq!(word.resource(), HyphenationResource::EnUs2005);
    assert_eq!(
        word.provenance(),
        Some(HyphenationProvenance::SourceException { entry: "associate" })
    );
    assert_eq!(
        word.cuts()
            .map(|cut| {
                assert!(cut.opportunity().is_some());
                let counts = cut.counts().unwrap();
                (
                    cut.boundary().view().source_before().get(),
                    counts.total,
                    counts.before,
                    counts.after,
                )
            })
            .collect::<Vec<_>>(),
        [(7, 9, 2, 7), (9, 9, 4, 5)]
    );
    let forced = p
        .opportunity_views()
        .filter(|offer| offer.kind() == PreparedBreakKind::Forced)
        .map(|offer| offer.boundary().view().source_before().get())
        .collect::<Vec<_>>();
    assert_eq!(
        forced,
        text.bytes()
            .enumerate()
            .filter_map(|(i, byte)| (byte == b'\n').then_some(i + 1))
            .collect::<Vec<_>>()
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                hyphenate_limit_last: policy,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture =
        LayoutFlowInput::try_new(&tree, 0, vec![paragraph.binding(0)], Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(100.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(layout.lines().len(), expected.len());
    for (index, (line, (range, advance, ending))) in layout.lines().iter().zip(expected).enumerate()
    {
        assert_eq!(line.range(), *range);
        assert_eq!(line.baseline(), 8.0 + 10.0 * index as f64);
        assert_eq!(
            line.bounds(),
            surgeist_text::Rect::new(0.0, 10.0 * index as f64, 100.0, 10.0)
        );
        // Empty terminal geometry is checked above without fabricating a text
        // fragment or requiring a source-bearing glyph for that empty line.
        if range.is_empty() {
            continue;
        }
        let finalized = batch
            .final_inline_fragments()
            .iter()
            .find(|row| {
                row.owner() == &crate::InlineFragmentOwner::Node(1)
                    && row.fragment().line_index() == index
            })
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(finalized.prepared_text(), paragraph.prepared);
        assert_eq!(finalized.feedback().advance, *advance);
        assert_eq!(finalized.feedback().line_height, 10.0);
        assert_eq!(finalized.selected_line().request().context.end, *ending);
        let selected = finalized.selected_line();
        let alternative = &selected.request().alternative;
        if index == 0 && (policy == HyphenateLimitLast::None || text.ends_with('Y')) {
            assert!(matches!(
                alternative.as_ref().unwrap().view().definition(),
                PreparedAlternativeDefinition::Automatic { .. }
            ));
        } else {
            assert!(alternative.is_none());
        }
        assert!(finalized.run_views().all(|run| run.face().bytes() == AHEM
            && run.effective_instance().unwrap().font_size() == 10.0));
    }
}

fn check_case<S: LayoutScalar>(
    text: &str,
    y_spacing: f64,
    ordinary: &[(Range, f64, LineEnd)],
    protected: &[(Range, f64, LineEnd)],
) {
    let paragraph = paragraph(text, y_spacing);
    // The same immutable preparation is consumed before, during and after tail
    // restriction. The changed block policy cannot mutate its resource offers
    // or carry branch exclusions into a later independent computation.
    for policy in [
        HyphenateLimitLast::None,
        HyphenateLimitLast::Always,
        HyphenateLimitLast::None,
    ] {
        check::<S>(
            &paragraph,
            text,
            policy,
            if policy == HyphenateLimitLast::Always {
                protected
            } else {
                ordinary
            },
        );
    }
}

#[test]
fn last_tail_preserves_empty_geometry_and_distinguishes_forced_segments_and_nonpositive_content() {
    for (text, ordinary, protected) in [
        (
            "XXXX associate",
            vec![
                (Range::new(0, 9), 100.0, LineEnd::Soft),
                (Range::new(9, 14), 50.0, LineEnd::Paragraph),
            ],
            vec![
                (Range::new(0, 5), 40.0, LineEnd::Soft),
                (Range::new(5, 14), 90.0, LineEnd::Paragraph),
            ],
        ),
        (
            "XXXX associate\n",
            vec![
                (Range::new(0, 9), 100.0, LineEnd::Soft),
                (Range::new(9, 15), 50.0, LineEnd::Forced),
                (Range::new(15, 15), 0.0, LineEnd::Paragraph),
            ],
            vec![
                (Range::new(0, 5), 40.0, LineEnd::Soft),
                (Range::new(5, 15), 90.0, LineEnd::Forced),
                (Range::new(15, 15), 0.0, LineEnd::Paragraph),
            ],
        ),
        (
            "XXXX associate\n\n",
            vec![
                (Range::new(0, 9), 100.0, LineEnd::Soft),
                (Range::new(9, 15), 50.0, LineEnd::Forced),
                (Range::new(15, 16), 0.0, LineEnd::Forced),
                (Range::new(16, 16), 0.0, LineEnd::Paragraph),
            ],
            vec![
                (Range::new(0, 5), 40.0, LineEnd::Soft),
                (Range::new(5, 15), 90.0, LineEnd::Forced),
                (Range::new(15, 16), 0.0, LineEnd::Forced),
                (Range::new(16, 16), 0.0, LineEnd::Paragraph),
            ],
        ),
    ] {
        check_case::<f32>(text, 0.0, &ordinary, &protected);
        check_case::<f64>(text, 0.0, &ordinary, &protected);
    }
    for spacing in [0.0, -10.0, -15.0] {
        // Retained Y is real content at advance10/0/−5; its Forced predecessor
        // prevents the earlier segment's Auto from becoming the protected tail.
        let unchanged = [
            (Range::new(0, 9), 100.0, LineEnd::Soft),
            (Range::new(9, 15), 50.0, LineEnd::Forced),
            (Range::new(15, 16), 10.0 + spacing, LineEnd::Paragraph),
        ];
        check_case::<f32>("XXXX associate\nY", spacing, &unchanged, &unchanged);
        check_case::<f64>("XXXX associate\nY", spacing, &unchanged, &unchanged);
    }
}

fn check_choice_order<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
    style.white_space = WhiteSpace::Collapse;
    style.wrap = Wrap::Word;
    style.overflow_wrap = OverflowWrap::Anywhere;
    let mut source = Source::new("X cooperate");
    source.span(
        Range::new(0, 2),
        surgeist_text::Style {
            letter_spacing: -10.0,
            ..style.clone()
        },
    );
    let paragraph = PreparedParagraph::plain(1, source, style);
    let prepared = &paragraph.prepared;
    let word = prepared
        .word_views()
        .find(|w| w.query() == Some("cooperate"))
        .unwrap();
    assert_eq!(word.resource(), HyphenationResource::EnUs2005);
    assert_eq!(word.provenance(), Some(HyphenationProvenance::Pattern));
    assert_eq!(
        word.cuts()
            .map(|cut| {
                assert!(cut.opportunity().is_some());
                let counts = cut.counts().unwrap();
                (
                    cut.boundary().view().source_before().get(),
                    counts.total,
                    counts.before,
                    counts.after,
                )
            })
            .collect::<Vec<_>>(),
        [(4, 9, 2, 7), (6, 9, 4, 5), (8, 9, 6, 3)]
    );
    let start = prepared.boundary_views().next().unwrap().handle();
    assert!(
        word.ordinary_fallbacks(&start)
            .unwrap()
            .iter()
            .any(|offer| offer.view().boundary().view().source_before().get() == 2)
    );
    for policy in [
        HyphenateLimitLast::None,
        HyphenateLimitLast::Always,
        HyphenateLimitLast::None,
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    hyphenate_limit_last: policy,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(60.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1));
        let capture =
            LayoutFlowInput::try_new(&tree, 0, vec![paragraph.binding(0)], Vec::new()).unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request(60.0)).unwrap();
        let layout = batch.placed_text()[0].layout();
        assert_eq!(
            layout.source().text().code_units(),
            "X cooperate".encode_utf16().collect::<Vec<_>>()
        );
        // Earlier eligible Auto2 outranks the zero-width ordinary fallback.
        // Auto4 is tried first, but its two-line result has a prohibited final
        // predecessor. Auto2 admits the later real nonhyphenating emergency
        // break; globally disabling all first-line hyphens would skip it.
        let expected = if policy == HyphenateLimitLast::Always {
            vec![
                (Range::new(0, 4), 30.0),
                (Range::new(4, 10), 60.0),
                (Range::new(10, 11), 10.0),
            ]
        } else {
            vec![(Range::new(0, 6), 50.0), (Range::new(6, 11), 50.0)]
        };
        assert_eq!(layout.lines().len(), expected.len());
        for (index, (line, (range, advance))) in layout.lines().iter().zip(&expected).enumerate() {
            assert_eq!(line.range(), *range);
            assert_eq!(line.baseline(), 8.0 + 10.0 * index as f64);
            assert_eq!(
                line.bounds(),
                surgeist_text::Rect::new(0.0, 10.0 * index as f64, 60.0, 10.0)
            );
            let finalized = batch
                .final_inline_fragments()
                .iter()
                .find(|row| {
                    row.owner() == &crate::InlineFragmentOwner::Node(1)
                        && row.fragment().line_index() == index
                })
                .unwrap()
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(finalized.prepared_text(), paragraph.prepared);
            assert_eq!(finalized.feedback().advance, *advance);
            let selected = finalized.selected_line();
            if index == 0 {
                let PreparedAlternativeDefinition::Automatic { cut } = selected
                    .request()
                    .alternative
                    .as_ref()
                    .unwrap()
                    .view()
                    .definition()
                else {
                    panic!("first ending retains a genuine automatic cut");
                };
                assert_eq!(
                    cut.view().counts().unwrap().before,
                    if policy == HyphenateLimitLast::Always {
                        2
                    } else {
                        4
                    }
                );
            } else {
                assert!(selected.request().alternative.is_none());
                if policy == HyphenateLimitLast::Always && index == 1 {
                    assert!(matches!(
                        selected.request().termination,
                        surgeist_text::PreparedTermination::Conditional(_)
                    ));
                }
            }
            assert!(finalized.run_views().all(|run| run.face().bytes() == AHEM
                && run.effective_instance().unwrap().font_size() == 10.0));
        }
    }
}

#[test]
fn last_tail_search_retains_an_earlier_hyphenated_choice_before_disabling_all_hyphenation() {
    check_choice_order::<f32>();
    check_choice_order::<f64>();
}
