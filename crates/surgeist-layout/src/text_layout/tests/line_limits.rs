//! Successive-line policy through real block computation and retained Text.
use super::*;
use surgeist_text::{HyphenateCharacter, Hyphens, OverflowWrap, PreparedTermination, Range, Text};

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
fn paragraph(source: &str, indication: &str, overflow: OverflowWrap) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Auto;
    style.white_space = surgeist_text::WhiteSpace::Collapse;
    style.locale = Some("en-US".into());
    style.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
    style.overflow_wrap = overflow;
    PreparedParagraph::plain(1, Source::new(source), style)
}
fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    limit: HyphenateLimitLines,
    width: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let binding = paragraph.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                hyphenate_limit_lines: limit,
                ..NodeInputOf::default()
            },
        );
    for origin in &binding.origins {
        tree = tree
            .children(origin.node, [])
            .input(origin.node, paragraph.text_input(origin.node));
    }
    for control in &binding.controls {
        tree = tree.children(control.node, []).input(
            control.node,
            LayoutInputOf::text_control(TextControlParticipation::new(control.occurrence.clone())),
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let input = LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MaxContent)).unwrap();
    compute_layout_with_flow(&capture, 0, input).unwrap()
}
fn observations<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
) -> Vec<(usize, f64, bool)> {
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        paragraph
            .prepared
            .analyzed_source()
            .source()
            .text()
            .code_units()
    );
    let mut lines = vec![None; layout.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let line = line.expect("source-backed committed line");
            assert!(line.run_views().all(|run| run.face().bytes() == AHEM));
            let selected = line.selected_line();
            assert_eq!(layout.lines()[index].baseline(), 8.0 + index as f64 * 10.0);
            (
                selected.request().end.view().source_before().get(),
                line.feedback().advance,
                selected.request().alternative.is_some(),
            )
        })
        .collect()
}
#[test]
fn finite_line_limits_suppress_only_automatic_endings_and_keep_manual_empty_custody() {
    fn check<S: LayoutScalar>() {
        let p = paragraph("cooperate", "A", OverflowWrap::Normal);
        let words = p.prepared.word_views().collect::<Vec<_>>();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].query(), Some("cooperate"));
        let cuts = words[0]
            .cuts()
            .map(|c| c.boundary().view().source_before().get())
            .collect::<Vec<_>>();
        assert_eq!(cuts, [2, 4, 6]);
        let unbroken = compute::<S>(
            &p,
            HyphenateLimitLines::NoLimit,
            AvailableOf::definite(S::from_f64(100.0)),
        );
        assert_eq!(observations(&unbroken, &p), [(9, 90.0, false)]);
        for (policy, expected) in [
            (
                HyphenateLimitLines::NoLimit,
                vec![
                    (2, 30.0, true),
                    (4, 30.0, true),
                    (6, 30.0, true),
                    (9, 30.0, false),
                ],
            ),
            (
                HyphenateLimitLines::Count(usize::MAX),
                vec![
                    (2, 30.0, true),
                    (4, 30.0, true),
                    (6, 30.0, true),
                    (9, 30.0, false),
                ],
            ),
            (
                HyphenateLimitLines::Count(1),
                vec![(2, 30.0, true), (9, 70.0, false)],
            ),
            (HyphenateLimitLines::Count(0), vec![(9, 90.0, false)]),
        ] {
            let batch = compute::<S>(&p, policy, AvailableOf::definite(S::from_f64(30.0)));
            assert_eq!(observations(&batch, &p), expected);
        }
        for indication in ["A", ""] {
            let p = paragraph("co\u{ad}operate", indication, OverflowWrap::Normal);
            let extra = if indication.is_empty() { 0.0 } else { 10.0 };
            let batch = compute::<S>(
                &p,
                HyphenateLimitLines::Count(1),
                AvailableOf::definite(S::from_f64(30.0)),
            );
            assert_eq!(
                observations(&batch, &p),
                [(3, 20.0 + extra, true), (10, 70.0, false)]
            );
            let p = paragraph(
                "co\u{ad}op\u{ad}er\u{ad}ate",
                indication,
                OverflowWrap::Normal,
            );
            let batch = compute::<S>(
                &p,
                HyphenateLimitLines::Count(0),
                AvailableOf::definite(S::from_f64(30.0)),
            );
            assert_eq!(
                observations(&batch, &p),
                [
                    (3, 20.0 + extra, true),
                    (6, 20.0 + extra, true),
                    (9, 20.0 + extra, true),
                    (12, 30.0, false)
                ]
            );
        }
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn ordinary_literal_and_emergency_endings_reset_the_committed_sequence() {
    fn check<S: LayoutScalar>() {
        for (source, overflow, expected) in [
            (
                "co\u{ad}operate cooperate",
                OverflowWrap::Normal,
                vec![
                    (3, 30.0, true),
                    (11, 70.0, false),
                    (13, 30.0, true),
                    (20, 70.0, false),
                ],
            ),
            (
                "co\u{ad}operate-cooperate",
                OverflowWrap::Normal,
                vec![
                    (3, 30.0, true),
                    (11, 80.0, false),
                    (13, 30.0, true),
                    (20, 70.0, false),
                ],
            ),
            (
                "co\u{ad}operate",
                OverflowWrap::Anywhere,
                vec![
                    (3, 30.0, true),
                    (6, 30.0, false),
                    (7, 20.0, true),
                    (10, 30.0, false),
                ],
            ),
            (
                "co\u{ad}operate",
                OverflowWrap::BreakWord,
                vec![
                    (3, 30.0, true),
                    (6, 30.0, false),
                    (7, 20.0, true),
                    (10, 30.0, false),
                ],
            ),
        ] {
            let p = paragraph(source, "A", overflow);
            let batch = compute::<S>(
                &p,
                HyphenateLimitLines::Count(1),
                AvailableOf::definite(S::from_f64(30.0)),
            );
            assert_eq!(observations(&batch, &p), expected, "{source:?}");
            if overflow != OverflowWrap::Normal {
                let fragment = batch
                    .final_inline_fragments()
                    .iter()
                    .find(|f| f.fragment().line_index() == 1)
                    .unwrap()
                    .fragment()
                    .finalized_fragment()
                    .finalized_line();
                assert!(matches!(
                    fragment.selected_line().request().termination,
                    PreparedTermination::Conditional(_)
                ));
                assert_eq!(
                    batch.placed_text()[0].layout().lines()[1].range(),
                    Range::new(3, 6)
                );
            }
        }
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn intrinsic_line_limits_are_static_and_independent_of_actual_query_order() {
    fn check<S: LayoutScalar>() {
        for source in ["as\u{ad}sociate", "associate"] {
            let p = paragraph(source, "A", OverflowWrap::Normal);
            for limit in [
                HyphenateLimitLines::Count(0),
                HyphenateLimitLines::Count(1),
                HyphenateLimitLines::NoLimit,
            ] {
                let actual = observations(
                    &compute::<S>(&p, limit, AvailableOf::definite(S::from_f64(30.0))),
                    &p,
                );
                for minimum in [false, true, true, false] {
                    let batch = compute::<S>(
                        &p,
                        limit,
                        if minimum {
                            AvailableOf::MinContent
                        } else {
                            AvailableOf::MaxContent
                        },
                    );
                    let width = batch
                        .unrounded_entries()
                        .iter()
                        .find(|entry| entry.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64();
                    let expected = if !minimum {
                        90.0
                    } else if source.contains('\u{ad}') && limit == HyphenateLimitLines::Count(0) {
                        70.0
                    } else if limit == HyphenateLimitLines::Count(0) {
                        90.0
                    } else {
                        50.0
                    };
                    assert_eq!(width, expected, "{source:?} {limit:?} minimum={minimum}");
                }
                assert_eq!(
                    observations(
                        &compute::<S>(&p, limit, AvailableOf::definite(S::from_f64(30.0))),
                        &p
                    ),
                    actual
                );
            }
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn real_forced_and_source_less_discretionary_endings_use_their_actual_roles() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
        style.white_space = surgeist_text::WhiteSpace::Preserve;
        let forced =
            PreparedParagraph::plain(1, Source::new("co\u{ad}operate\ncooperate"), style.clone());
        let batch = compute::<S>(
            &forced,
            HyphenateLimitLines::Count(1),
            AvailableOf::definite(S::from_f64(30.0)),
        );
        assert_eq!(
            observations(&batch, &forced),
            [
                (3, 30.0, true),
                (11, 70.0, false),
                (13, 30.0, true),
                (20, 70.0, false)
            ]
        );
        let lf = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.fragment().line_index() == 1)
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(
            lf.selected_line().request().context.end,
            surgeist_text::LineEnd::Forced
        );
        assert!(
            matches!(&lf.selected_line().request().termination,PreparedTermination::Ordinary(offer) if offer.view().kind()==surgeist_text::PreparedBreakKind::Forced)
        );
        for indication in ["A", ""] {
            style.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
            let control = PreparedParagraph::new(
                vec![
                    ParagraphPart::text(1, Source::new("co"), style.clone()),
                    ParagraphPart::Control {
                        node: 2,
                        kind: ParagraphControlKind::DiscretionaryBreak {
                            style: Box::new(style.clone()),
                        },
                    },
                    ParagraphPart::text(3, Source::new("operate"), style.clone()),
                ],
                style.clone(),
                ParagraphBaseDirection::LeftToRight,
            );
            let batch = compute::<S>(
                &control,
                HyphenateLimitLines::Count(1),
                AvailableOf::definite(S::from_f64(30.0)),
            );
            assert_eq!(
                observations(&batch, &control),
                [
                    (2, if indication.is_empty() { 20.0 } else { 30.0 }, true),
                    (9, 70.0, false)
                ]
            );
            let first = batch
                .final_inline_fragments()
                .iter()
                .find(|entry| entry.fragment().line_index() == 0)
                .unwrap()
                .fragment()
                .finalized_fragment()
                .finalized_line();
            let selected = first.selected_line();
            let alternative = selected.request().alternative.as_ref().unwrap();
            assert!(matches!(
                alternative.view().definition(),
                surgeist_text::PreparedAlternativeDefinition::Control { .. }
            ));
            assert_eq!(
                alternative.view().text().code_units(),
                indication.encode_utf16().collect::<Vec<_>>()
            );
        }
    }
    check::<f32>();
    check::<f64>();
}
