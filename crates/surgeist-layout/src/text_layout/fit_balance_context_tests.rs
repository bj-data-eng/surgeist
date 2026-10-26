//! Balance changes endpoints, never the real tab/percentage frame or stage authority.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use std::cell::RefCell;
use surgeist_text as text;

const PHRASE_FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf"
));

fn ordinary_facts(lines: &[FinalizedLine]) -> Vec<(usize, f64)> {
    lines
        .iter()
        .map(|line| {
            let selected = line.selected_line();
            let request = selected.request();
            assert_eq!(request.context.available_inline, Some(60.0));
            assert!(
                request.alternative.is_none(),
                "winning whole-word program: {request:?}"
            );
            assert!(
                request.incoming.is_none(),
                "winning whole-word program: {request:?}"
            );
            let PreparedTermination::Ordinary(offer) = &request.termination else {
                panic!("a real fitting ordinary candidate cannot authorize a conditional stage");
            };
            assert_eq!(offer.view().boundary(), request.end);
            assert_eq!(
                offer.view().boundary().prepared_text(),
                line.prepared_text()
            );
            assert_eq!(
                offer.view().kind(),
                if request.context.end == LineEnd::Soft {
                    PreparedBreakKind::Allowed
                } else {
                    PreparedBreakKind::ParagraphEnd
                }
            );
            (
                request.end.view().source_after().get(),
                line.measurements().alignment().advance(),
            )
        })
        .collect()
}

fn tab_batch(
    paragraph: &PreparedParagraph,
    wrap_style: crate::TextWrapStyle,
) -> crate::CompletedLayoutBatchOf<u32, f64> {
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            crate::NodeInputOf {
                display: crate::Display::Block,
                text_wrap_style: wrap_style,
                text_indent: crate::TextIndentOf {
                    // Exercise the real block resolver: 25% of actual 60 = 15.
                    amount: crate::LengthPercentageOf::from_percent_fraction(0.25).unwrap(),
                    each_line: false,
                    hanging: false,
                },
                size: crate::Size::new(
                    crate::PreferredSizeOf::value(crate::LengthPercentageOf::px(60.0).unwrap()),
                    crate::PreferredSizeOf::AUTO,
                ),
                ..crate::NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let flow = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput::default(),
    )
    .unwrap();
    crate::compute_layout_with_flow(
        &flow,
        0,
        crate::LayoutRootRequestOf::viewport(crate::Size::new(
            crate::AvailableOf::definite(60.0),
            crate::AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn published_lines(batch: &crate::CompletedLayoutBatchOf<u32, f64>) -> Vec<FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        if let Some(old) = &lines[fragment.line_index()] {
            assert_eq!(old, &line);
        }
        lines[fragment.line_index()] = Some(line);
    }
    lines
        .into_iter()
        .map(|line| line.expect("each real body line retains an authentic origin fragment"))
        .collect()
}

#[test]
fn alternate_tab_cut_keeps_actual_css_grid_and_percentage_containing_width() {
    // At first-line x15, A ends25, the genuine10px tab ends30, and B ends40.
    // Auto also fits SPACE+C through60: natural45, then D10 (residual0/50).
    // Balance cuts after B: natural25, then C SPACE D30 (residual20/30).
    // Population variance625 ->25, still2 lines. Every request retains60,
    // first indent15 from the actual percentage resolver, and tab origin0.
    let base = text::Style {
        white_space: text::WhiteSpace::Collapse,
        wrap: text::Wrap::Word,
        word_break: text::WordBreak::Normal,
        hyphens: text::Hyphens::None,
        tab_size: text::TabSize::length(10.0).unwrap(),
        ..ahem_style(10.0, 10.0)
    };
    let mut source = text::Source::new("A\tB C D");
    source.span(
        text::Range::new(1, 2),
        text::Style {
            white_space: text::WhiteSpace::Preserve,
            ..base.clone()
        },
    );
    let paragraph = PreparedParagraph::with_css_tabs(
        vec![ParagraphPart::text(1, source, base.clone())],
        base,
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(
        paragraph.prepared.preparation_options().tabs(),
        TabPreparation::CssStops
    );
    assert!(paragraph.prepared.tab_reference().is_some());
    let auto = tab_batch(&paragraph, crate::TextWrapStyle::Auto);
    let balanced = tab_batch(&paragraph, crate::TextWrapStyle::Balance);
    let auto_lines = published_lines(&auto);
    let balanced_lines = published_lines(&balanced);
    assert_eq!(ordinary_facts(&auto_lines), [(6, 45.0), (7, 10.0)]);
    assert_eq!(ordinary_facts(&balanced_lines), [(4, 25.0), (7, 30.0)]);
    for (batch, lines) in [(&auto, &auto_lines), (&balanced, &balanced_lines)] {
        assert_eq!(
            batch.placed_text()[0].layout().source().text(),
            &text::Text::from("A\tB C D")
        );
        for (index, line) in lines.iter().enumerate() {
            assert_eq!(line.prepared_text(), paragraph.prepared);
            let selected = line.selected_line();
            assert_eq!(
                selected.request().context.indent,
                if index == 0 { 15.0 } else { 0.0 }
            );
            assert_eq!(
                selected.request().context.tabs,
                TabContext::CssStops { origin: Some(0.0) }
            );
        }
        let positions = batch.placed_text()[0]
            .layout()
            .run_views()
            .filter(|run| run.line() == 0)
            .flat_map(|run| {
                run.glyphs()
                    .filter(|glyph| glyph.id() == 35 || glyph.id() == 36)
                    .map(|glyph| (glyph.id(), run.origin().x + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(positions, [(35, 15.0), (36, 30.0)]);
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, crate::Size::new(60.0, 20.0));
    }
    assert_eq!(
        ordinary_facts(&published_lines(&auto)),
        [(6, 45.0), (7, 10.0)]
    );
}

fn fitting_ordinary_trials(
    paragraph: &PreparedParagraph,
    wrap_style: crate::TextWrapStyle,
) -> Vec<FinalizedLine> {
    let observed = RefCell::new(Vec::<LineSelectionRequest>::new());
    let lines = super::super::fit(
        FitInput {
            wrapping: None,
            formatted: &FormattedTailSnapshot::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                crate::Direction::Ltr,
            ),
            prepared: &paragraph.prepared,
            available: Available::Definite(60.0),
            base_direction: ParagraphBaseDirection::LeftToRight,
            block_direction: ParagraphBaseDirection::LeftToRight,
            alignment: TextAlign::Start,
            wrap_style,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: TailContext::Unrestricted,
            orientation: LineOrientation::Horizontal,
            atomics: &[],
            controls: &[],
            line_strut: None,
            reusable: &[],
        },
        |_, _| {
            Ok(Band {
                start: 0.0,
                end: 60.0,
                intrinsic_end_occupation: None,
                next_transition: None,
            })
        },
        |position, _| {
            Ok(position
                .minimum
                .map_or(position.proposed, |minimum| position.proposed.max(minimum)))
        },
        |_, _, _, struts| {
            Ok(LineAllocations {
                line_box_struts: struts.to_vec(),
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            })
        },
        |_, _| Ok(None),
        EncounterCallbacks {
            replay: |line: &FinalizedLine, _: EncounterReplay| {
                // Observe authentic requests of every provisional/replayed trial,
                // including losing goals. No call count or manufactured offer.
                assert_eq!(line.prepared_text(), paragraph.prepared);
                observed
                    .borrow_mut()
                    .push(line.selected_line().request().clone());
                Ok(Vec::new())
            },
            accept: || Ok(()),
            reset: |_| {},
            checkpoint_group: || {},
        },
    )
    .unwrap_or_else(|error| match error {
        FitError::Text(error) => panic!("real Text producer error: {error:?}"),
        FitError::Wrapping(error) => panic!("real wrapping input error: {error:?}"),
        other => panic!("actual fitter failed: {:?}", std::mem::discriminant(&other)),
    });
    let observed = observed.into_inner();
    assert!(!observed.is_empty());
    for request in observed {
        assert_eq!(
            request.context.available_inline,
            Some(60.0),
            "{wrap_style:?}: {request:?}"
        );
        assert_eq!(request.context.indent, 0.0, "{wrap_style:?}: {request:?}");
        let PreparedTermination::Ordinary(offer) = &request.termination else {
            panic!("{wrap_style:?}: a goal is not real overflow authority: {request:?}");
        };
        assert!(
            paragraph
                .prepared
                .opportunity_views()
                .any(|view| view.handle() == *offer),
            "{wrap_style:?}: {request:?}"
        );
        assert_eq!(
            offer.view().boundary(),
            request.end,
            "{wrap_style:?}: {request:?}"
        );
        if let Some(alternative) = &request.alternative {
            // Declared Hyphens::Auto produces real ordinary Discretionary
            // alternatives. Their exploration is not conditional-stage entry.
            assert_eq!(
                alternative.view().availability(),
                text::PreparedAlternativeAvailability::Ordinary,
                "{wrap_style:?}: {request:?}"
            );
            assert_eq!(
                alternative.view().boundary(),
                request.end,
                "{wrap_style:?}: {request:?}"
            );
            assert!(
                offer
                    .view()
                    .alternatives()
                    .any(|view| view.handle() == *alternative),
                "{wrap_style:?}: {request:?}"
            );
        }
        if let Some(incoming) = &request.incoming {
            // Text authenticates exact receiving custody when it finalizes
            // this request; retain the genuine originating ordinary offer.
            let alternative = incoming.alternative();
            assert_eq!(
                incoming.boundary(),
                request.start,
                "{wrap_style:?}: {request:?}"
            );
            assert_eq!(
                alternative.view().boundary(),
                request.start,
                "{wrap_style:?}: {request:?}"
            );
            assert_eq!(
                alternative.view().availability(),
                text::PreparedAlternativeAvailability::Ordinary,
                "{wrap_style:?}: {request:?}"
            );
            assert!(
                paragraph
                    .prepared
                    .opportunity_views()
                    .any(|view| view.boundary() == request.start
                        && view
                            .alternatives()
                            .any(|candidate| candidate.handle() == alternative)),
                "{wrap_style:?}: {request:?}"
            );
        }
    }
    lines.into_iter().map(|line| line.finalized).collect()
}

#[test]
fn balancing_real_phrase_suffix_never_restores_normal_or_emergency_while_ordinary_fits() {
    // Original Ahem10 widths: AA SPACE A SPACE A =60, actual Japanese phrase=30.
    // Auto cut7 gives60/30 (residual0/30, variance225). Balance cut5 gives
    // AA SPACE A =40 and A SPACE phrase=50 (20/10, variance25).
    // The whole30px phrase fits real60 at every reachable ordinary prefix.
    let japanese = text::Style {
        font: text::Font::new().family("Surgeist Phrase Ahem"),
        locale: Some("ja".into()),
        word_break: text::WordBreak::AutoPhrase,
        overflow_wrap: text::OverflowWrap::Anywhere,
        white_space: text::WhiteSpace::Collapse,
        wrap: text::Wrap::Word,
        hyphens: text::Hyphens::None,
        ..ahem_style(10.0, 10.0)
    };
    let prefix = text::Style {
        locale: Some("en-US".into()),
        word_break: text::WordBreak::Normal,
        ..japanese.clone()
    };
    let paragraph = PreparedParagraph::with_registered_font(
        vec![
            ParagraphPart::text(1, text::Source::new("AA A A "), prefix),
            ParagraphPart::text(2, text::Source::new("日本へ"), japanese.clone()),
        ],
        japanese,
        ParagraphBaseDirection::LeftToRight,
        PHRASE_FONT,
    );
    assert_eq!(paragraph.prepared.phrases().count(), 1);
    let auto = fitting_ordinary_trials(&paragraph, crate::TextWrapStyle::Auto);
    let balanced = fitting_ordinary_trials(&paragraph, crate::TextWrapStyle::Balance);
    assert_eq!(ordinary_facts(&auto), [(7, 60.0), (10, 30.0)]);
    assert_eq!(ordinary_facts(&balanced), [(5, 40.0), (10, 50.0)]);
    assert_eq!(ordinary_facts(&auto), [(7, 60.0), (10, 30.0)]);
}

#[test]
fn balancing_fitting_words_keeps_ordinary_hyphen_trials_out_of_conditional_stages() {
    // Ahem5: first AAA... word30, each A5, each retained SPACE5, associate45.
    // Auto first AAAAAA SPACE A SPACE A SPACE A =60 (cut13), last45.
    // Balance first AAAAAA SPACE A SPACE A =50 (cut11), last A SPACE associate55.
    // Residual0/15 variance56.25 ->10/5 variance6.25; all whole words fit60.
    let style = text::Style {
        white_space: text::WhiteSpace::Collapse,
        wrap: text::Wrap::Word,
        word_break: text::WordBreak::Normal,
        overflow_wrap: text::OverflowWrap::Anywhere,
        hyphens: text::Hyphens::Auto,
        locale: Some("en-US".into()),
        hyphenate_character: text::HyphenateCharacter::String("A".into()),
        ..ahem_style(5.0, 10.0)
    };
    let paragraph = PreparedParagraph::plain(1, text::Source::new("AAAAAA A A A associate"), style);
    let word = paragraph
        .prepared
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    assert!(
        word.cuts().next().is_some(),
        "real owned automatic-hyphenation proposals are present"
    );
    assert!(
        paragraph
            .prepared
            .opportunity_views()
            .any(|offer| offer.kind() == PreparedBreakKind::Discretionary
                && offer.alternatives().any(|alternative| matches!(
                    alternative.definition(),
                    text::PreparedAlternativeDefinition::Automatic { .. }
                ))),
        "the fixture has actual ordinary automatic-hyphen alternatives, not only a dormant resource"
    );
    let auto = fitting_ordinary_trials(&paragraph, crate::TextWrapStyle::Auto);
    let balanced = fitting_ordinary_trials(&paragraph, crate::TextWrapStyle::Balance);
    assert_eq!(ordinary_facts(&auto), [(13, 60.0), (22, 45.0)]);
    assert_eq!(ordinary_facts(&balanced), [(11, 50.0), (22, 55.0)]);
    assert_eq!(ordinary_facts(&auto), [(13, 60.0), (22, 45.0)]);
}
