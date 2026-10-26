//! Real candidate ordering and branch-local terminal reselection.
use super::*;
use surgeist_text::{HyphenateCharacter, Hyphens, OverflowWrap, Range, Text, WhiteSpace};

fn paragraph(text: &str, indication: &str, overflow: OverflowWrap) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Manual;
    style.white_space = WhiteSpace::Collapse;
    style.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
    style.overflow_wrap = overflow;
    PreparedParagraph::plain(1, Source::new(text), style)
}
fn lines<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: f64,
    policy: HyphenateLimitLast,
) -> Vec<(Range, f64, bool)> {
    let batch = batch::<S>(p, AvailableOf::definite(S::from_f64(width)), policy);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        p.prepared.analyzed_source().source().text().code_units()
    );
    let mut observed = vec![None; layout.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        let selected = line.selected_line();
        assert!(line.has_in_flow_content());
        assert!(line.run_views().all(|run| run.font_size() == 10.0));
        observed[fragment.line_index()] = Some((
            layout.lines()[fragment.line_index()].range(),
            line.feedback().advance,
            selected.request().alternative.is_some(),
        ));
    }
    observed.into_iter().map(Option::unwrap).collect()
}
#[test]
fn manual_later_ceiling_and_anti_permanent_ban_are_real_selected_sequences() {
    fn check<S: LayoutScalar>() {
        for indication in ["A", ""] {
            let p = paragraph("as\u{ad}sociate", indication, OverflowWrap::Anywhere);
            let manual = p
                .prepared
                .opportunity_views()
                .find(|offer| offer.kind() == surgeist_text::PreparedBreakKind::Discretionary)
                .unwrap();
            assert_eq!(manual.boundary().view().source_before().get(), 3);
            assert_eq!(manual.alternatives().count(), 1);
            assert_eq!(
                lines::<S>(&p, 70.0, HyphenateLimitLast::None),
                [
                    (
                        Range::new(0, 3),
                        if indication.is_empty() { 20.0 } else { 30.0 },
                        true
                    ),
                    (Range::new(3, 10), 70.0, false)
                ]
            );
            assert_eq!(
                lines::<S>(&p, 70.0, HyphenateLimitLast::Always),
                [
                    (Range::new(0, 8), 70.0, false),
                    (Range::new(8, 10), 20.0, false)
                ]
            );
            let normal = lines::<S>(&p, 50.0, HyphenateLimitLast::None);
            assert_eq!(
                normal,
                [
                    (
                        Range::new(0, 3),
                        if indication.is_empty() { 20.0 } else { 30.0 },
                        true
                    ),
                    (Range::new(3, 8), 50.0, false),
                    (Range::new(8, 10), 20.0, false)
                ]
            );
            assert_eq!(lines::<S>(&p, 50.0, HyphenateLimitLast::Always), normal);
            assert_eq!(lines::<S>(&p, 50.0, HyphenateLimitLast::None), normal);
        }
        let p = paragraph("as\u{ad}sociate", "A", OverflowWrap::Normal);
        assert_eq!(
            lines::<S>(&p, 70.0, HyphenateLimitLast::Always),
            [(Range::new(0, 10), 90.0, false)]
        );
    }
    check::<f32>();
    check::<f64>();
}

fn batch<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: AvailableOf<S>,
    policy: HyphenateLimitLast,
) -> CompletedLayoutBatchOf<u32, S> {
    let binding = p.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                hyphenate_limit_last: policy,
                ..NodeInputOf::default()
            },
        );
    for origin in &binding.origins {
        tree = tree
            .children(origin.node, [])
            .input(origin.node, p.text_input(origin.node));
    }
    for control in &binding.controls {
        tree = tree.children(control.node, []).input(
            control.node,
            LayoutInputOf::text_control(TextControlParticipation::new(control.occurrence.clone())),
        );
    }
    for atomic in &binding.atomics {
        let view = atomic.occurrence.view();
        let original = view.original_box();
        tree = tree.children(atomic.node, []).style(
            atomic.node,
            NodeInputOf {
                display: Display::InlineBlock,
                position: if original.kind() == surgeist_text::InlineBoxKind::OutOfFlow {
                    Position::Absolute
                } else {
                    Position::default()
                },
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(original.size().width)),
                    PreferredSizeOf::px(S::from_f64(original.size().height)),
                ),
                ..NodeInputOf::default()
            },
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
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MaxContent)).unwrap(),
    )
    .unwrap()
}

#[test]
fn source_control_empty_and_literal_endings_retain_their_actual_roles() {
    fn check<S: LayoutScalar>() {
        for indication in ["A", ""] {
            let source = paragraph("XXXX as\u{ad}sociate", indication, OverflowWrap::Normal);
            let mut style = ahem_style(10.0, 10.0);
            style.hyphens = Hyphens::Manual;
            style.white_space = WhiteSpace::Collapse;
            style.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
            let control = PreparedParagraph::new(
                vec![
                    ParagraphPart::text(1, Source::new("XXXX as"), style.clone()),
                    ParagraphPart::Control {
                        node: 2,
                        kind: ParagraphControlKind::DiscretionaryBreak {
                            style: Box::new(style.clone()),
                        },
                    },
                    ParagraphPart::text(3, Source::new("sociate"), style.clone()),
                ],
                style.clone(),
                ParagraphBaseDirection::LeftToRight,
            );
            for (p, cut, end) in [(&source, 8, 15), (&control, 7, 14)] {
                assert_eq!(
                    lines::<S>(p, 100.0, HyphenateLimitLast::None),
                    [
                        (
                            Range::new(0, cut),
                            if indication.is_empty() { 70.0 } else { 80.0 },
                            true
                        ),
                        (Range::new(cut, end), 70.0, false)
                    ]
                );
                assert_eq!(
                    lines::<S>(p, 100.0, HyphenateLimitLast::Always),
                    [
                        (Range::new(0, 5), 40.0, false),
                        (Range::new(5, end), 90.0, false)
                    ]
                );
            }
            style.hyphens = Hyphens::None;
            let literal = PreparedParagraph::plain(1, Source::new("XXXX asso-ciate"), style);
            assert_eq!(
                lines::<S>(&literal, 100.0, HyphenateLimitLast::Always),
                [
                    (Range::new(0, 10), 100.0, false),
                    (Range::new(10, 15), 50.0, false)
                ]
            );
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn real_zero_atomic_and_out_of_flow_tails_do_not_share_content_authority() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
        style.white_space = WhiteSpace::Collapse;
        let mut source = Source::new("XXXX associate\n");
        source.span(
            Range::new(14, 15),
            surgeist_text::Style {
                white_space: WhiteSpace::Preserve,
                ..style.clone()
            },
        );
        for inflow in [false, true] {
            let tail = if inflow {
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(0.0, 0.0),
                }
            } else {
                ParagraphPart::OutOfFlow {
                    node: 2,
                    size: surgeist_text::Size::new(0.0, 0.0),
                }
            };
            let p = PreparedParagraph::new(
                vec![ParagraphPart::text(1, source.clone(), style.clone()), tail],
                style.clone(),
                ParagraphBaseDirection::LeftToRight,
            );
            let batch = batch::<S>(
                &p,
                AvailableOf::definite(S::from_f64(100.0)),
                HyphenateLimitLast::Always,
            );
            let layout = batch.placed_text()[0].layout();
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                if inflow {
                    vec![Range::new(0, 9), Range::new(9, 15), Range::new(15, 15)]
                } else {
                    vec![Range::new(0, 5), Range::new(5, 15)]
                }
            );
            if inflow {
                assert_eq!(layout.nonformatted_lines().count(), 0);
                assert_eq!(layout.lines()[2].baseline(), 28.0);
                assert_eq!(layout.lines()[2].bounds().size.height, 10.0);
                let atomics = layout.inline_boxes();
                assert_eq!(atomics.len(), 1);
                assert_eq!(atomics[0].kind(), surgeist_text::InlineBoxKind::InFlow);
                assert_eq!(atomics[0].line(), 2);
                assert_eq!(atomics[0].rect().size, surgeist_text::Size::new(0.0, 0.0));
            } else {
                let completions = layout.nonformatted_lines().collect::<Vec<_>>();
                assert_eq!(completions.len(), 1);
                let completion = completions[0];
                assert_eq!(completion.origin, surgeist_text::Point::new(0.0, 20.0));
                let receipt = completion.line.finalized_line();
                assert_eq!(receipt.prepared_text(), p.prepared);
                assert!(!receipt.has_in_flow_content());
                let selected = receipt.selected_line();
                assert_eq!(
                    selected.request().context.end,
                    surgeist_text::LineEnd::Paragraph
                );
                assert_eq!(selected.request().atomics.len(), 1);
                assert!(matches!(
                    selected.request().atomics[0].allocation,
                    surgeist_text::AtomicLineAllocation::OutOfFlow
                ));
                assert_eq!(completion.atomics.len(), 1);
                assert_eq!(
                    completion.atomics[0].occurrence,
                    selected.request().atomics[0].occurrence
                );
                let native = batch
                    .final_entries()
                    .iter()
                    .find(|row| row.node() == 2)
                    .unwrap()
                    .output();
                assert_eq!(native.location, Point::new(S::ZERO, S::from_f64(20.0)));
                assert_eq!(native.size, Size::ZERO);
            }
            let first = batch
                .final_inline_fragments()
                .iter()
                .find(|entry| entry.fragment().line_index() == 0)
                .unwrap()
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(
                first.selected_line().request().alternative.is_some(),
                inflow
            );
            assert_eq!(first.feedback().advance, if inflow { 100.0 } else { 40.0 });
            assert_eq!(
                layout.source().text().code_units(),
                "XXXX associate\n".encode_utf16().collect::<Vec<_>>()
            );
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn intrinsic_contributions_do_not_construct_a_virtual_terminal_tail() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.white_space = WhiteSpace::Collapse;
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
        let p = PreparedParagraph::plain(1, Source::new("XXXX associate"), style);
        for policy in [
            HyphenateLimitLast::None,
            HyphenateLimitLast::Always,
            HyphenateLimitLast::None,
        ] {
            for (width, expected) in [
                (AvailableOf::MinContent, 50.0),
                (AvailableOf::MaxContent, 140.0),
            ] {
                let b = batch::<S>(&p, width, policy);
                assert_eq!(
                    b.final_entries()
                        .iter()
                        .find(|row| row.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64(),
                    expected
                );
            }
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn a_flow_box_does_not_invent_a_forced_barrier_for_missing_parent_continuation() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Manual;
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
        style.hyphenate_limit_chars.after = surgeist_text::HyphenationMinimum::Count(0);
        let first = PreparedParagraph::at_boundary(
            vec![ParagraphPart::text(
                1,
                Source::new("associate\u{ad}"),
                style.clone(),
            )],
            style.clone(),
            ParagraphBaseDirection::LeftToRight,
            surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
        );
        let tail = PreparedParagraph::plain(3, Source::new("Y"), style);
        let offer = first
            .prepared
            .opportunity_views()
            .find(|o| o.kind() == surgeist_text::PreparedBreakKind::Discretionary)
            .expect("actual trailing authored SHY admitted by real after=0 policy");
        assert_eq!(offer.boundary().view().source_before().get(), 10);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .input(1, first.text_input(1))
            .input(3, tail.text_input(3))
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    hyphenate_limit_last: HyphenateLimitLast::Always,
                    ..NodeInputOf::default()
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![first.binding(0), tail.binding(0)],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let error = compute_layout_with_flow(&capture, 0, request(80.0)).unwrap_err();
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::MissingContext(LayoutMissingContext::TextBlockTail)
        ));
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn genuine_forced_segment_and_ordinary_cross_run_results_need_no_missing_tail_proof() {
    fn check<S: LayoutScalar>() {
        for forced in [false, true] {
            let mut style = ahem_style(10.0, 10.0);
            style.hyphens = if forced {
                Hyphens::Manual
            } else {
                Hyphens::None
            };
            style.hyphenate_limit_chars.after = surgeist_text::HyphenationMinimum::Count(0);
            style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
            let mut source = Source::new(if forced {
                "as\u{ad}sociate\n"
            } else {
                "associate"
            });
            if forced {
                source.span(
                    Range::new(10, 11),
                    surgeist_text::Style {
                        white_space: WhiteSpace::Preserve,
                        ..style.clone()
                    },
                );
            }
            let first = PreparedParagraph::at_boundary(
                vec![ParagraphPart::text(1, source.clone(), style.clone())],
                style.clone(),
                ParagraphBaseDirection::LeftToRight,
                surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
            );
            let tail = PreparedParagraph::plain(3, Source::new("Y"), style);
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1, 2, 3])
                .children(1, [])
                .children(2, [])
                .children(3, [])
                .input(1, first.text_input(1))
                .input(3, tail.text_input(3))
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        hyphenate_limit_last: HyphenateLimitLast::Always,
                        ..NodeInputOf::default()
                    },
                );
            let capture = LayoutFlowInput::try_new(
                &tree,
                0,
                vec![first.binding(0), tail.binding(0)],
                crate::WrappingInput {
                    scopes: Vec::new(),
                    child_order: Vec::new(),
                },
            )
            .unwrap();
            let batch =
                compute_layout_with_flow(&capture, 0, request(if forced { 30.0 } else { 80.0 }))
                    .unwrap();
            assert_eq!(batch.placed_text().len(), 2);
            assert_eq!(
                batch.placed_text()[0].layout().source().text().code_units(),
                source.text().code_units()
            );
            let line = batch
                .final_inline_fragments()
                .iter()
                .find(|row| {
                    row.owner() == &crate::InlineFragmentOwner::Node(1)
                        && row.fragment().line_index() == 0
                })
                .unwrap()
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(line.selected_line().request().alternative.is_some(), forced);
            assert_eq!(line.feedback().advance, if forced { 30.0 } else { 90.0 });
            if forced {
                assert!(
                    first
                        .prepared
                        .opportunity_views()
                        .any(|o| o.kind() == surgeist_text::PreparedBreakKind::Forced
                            && o.boundary().view().source_before().get() == 11)
                );
            }
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn a_later_one_line_forced_segment_stays_independent_before_a_child_block_tail() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Manual;
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
        let mut source = Source::new("as\u{ad}sociate\nY");
        source.span(
            Range::new(10, 11),
            surgeist_text::Style {
                white_space: WhiteSpace::Preserve,
                ..style.clone()
            },
        );
        let p = PreparedParagraph::at_boundary(
            vec![ParagraphPart::text(1, source.clone(), style.clone())],
            style,
            ParagraphBaseDirection::LeftToRight,
            surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
        );
        assert!(p.prepared.opportunity_views().any(|o| o.kind()
            == surgeist_text::PreparedBreakKind::Discretionary
            && o.boundary().view().source_before().get() == 3));
        assert!(
            p.prepared
                .opportunity_views()
                .any(|o| o.kind() == surgeist_text::PreparedBreakKind::Forced
                    && o.boundary().view().source_before().get() == 11)
        );
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .input(1, p.text_input(1))
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    hyphenate_limit_last: HyphenateLimitLast::Always,
                    ..NodeInputOf::default()
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![p.binding(0)],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request(70.0)).unwrap();
        let layout = batch.placed_text()[0].layout();
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(3, 11), Range::new(11, 12)]
        );
        assert_eq!(
            layout.source().text().code_units(),
            source.text().code_units()
        );
        let line = batch
            .final_inline_fragments()
            .iter()
            .find(|row| row.fragment().line_index() == 0)
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(line.feedback().advance, 30.0);
        assert!(line.selected_line().request().alternative.is_some());
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn a_rejected_fitting_paragraph_cannot_manufacture_emergency_exhaustion() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.white_space = WhiteSpace::Collapse;
        style.overflow_wrap = OverflowWrap::Anywhere;
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
        let mut source = Source::new("cooperate");
        source.span(
            Range::new(5, 6),
            surgeist_text::Style {
                hyphenate_character: HyphenateCharacter::String(Text::from("AAAAAAAAAA")),
                ..style.clone()
            },
        );
        let p = PreparedParagraph::plain(1, source, style);
        let word = p
            .prepared
            .word_views()
            .find(|w| w.query() == Some("cooperate"))
            .unwrap();
        assert_eq!(
            word.cuts()
                .map(|c| c.boundary().view().source_before().get())
                .collect::<Vec<_>>(),
            [2, 4, 6]
        );
        let long = p
            .prepared
            .alternative_views()
            .find(|a| a.text().len() == 10)
            .unwrap();
        assert_eq!(
            long.opportunity()
                .expect("ordinary alternative")
                .view()
                .boundary()
                .view()
                .source_before()
                .get(),
            6
        );
        assert_eq!(
            lines::<S>(&p, 60.0, HyphenateLimitLast::None),
            [
                (Range::new(0, 4), 50.0, true),
                (Range::new(4, 9), 50.0, false)
            ]
        );
        assert_eq!(
            lines::<S>(&p, 60.0, HyphenateLimitLast::Always),
            [
                (Range::new(0, 2), 30.0, true),
                (Range::new(2, 8), 60.0, false),
                (Range::new(8, 9), 10.0, false)
            ]
        );
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn intrinsic_first_refinement_is_not_filtered_by_a_later_fitting_none() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.white_space = WhiteSpace::Collapse;
        style.letter_spacing = -10.0;
        style.word_spacing = -10.0;
        style.hyphenate_character = HyphenateCharacter::String(Text::from("A\u{a0}"));
        let p = PreparedParagraph::plain(1, Source::new("associate"), style);
        assert_eq!(
            p.prepared.analyzed_source().source().text().code_units(),
            "associate".encode_utf16().collect::<Vec<_>>()
        );
        let word = p
            .prepared
            .word_views()
            .find(|w| w.query() == Some("associate"))
            .unwrap();
        assert_eq!(
            word.resource(),
            surgeist_text::HyphenationResource::EnUs2005
        );
        assert_eq!(
            word.provenance(),
            Some(surgeist_text::HyphenationProvenance::SourceException { entry: "associate" })
        );
        let cut = word
            .cuts()
            .find(|c| c.counts().unwrap().before == 2)
            .unwrap();
        assert_eq!(cut.boundary().view().source_before().get(), 2);
        let offer = cut.opportunity().unwrap();
        let view = offer.view();
        let alternative = view.alternatives().next().unwrap().handle();
        assert_eq!(alternative.view().text().code_units(), [65, 0xa0]);
        let whole = crate::test_support::text_fragment::fragment_for(&p.prepared).finalized_line();
        let measure = whole.measurements().min_content_contribution();
        assert_eq!(
            (
                measure.advance(),
                measure.occupied_start(),
                measure.occupied_end()
            ),
            (0.0, 0.0, 0.0)
        );
        let mut request = whole.selected_line().request().clone();
        request.end = view.boundary();
        request.participants = p
            .prepared
            .participants()
            .take(request.end.view().participant_offset())
            .collect();
        request.termination = surgeist_text::PreparedTermination::Ordinary(offer.clone());
        request.alternative = Some(alternative);
        request.context.available_inline = None;
        request.context.end = surgeist_text::LineEnd::Soft;
        request.context.spacing = surgeist_text::SpacingPolicy::None;
        let first = p
            .prepared
            .select(request)
            .unwrap()
            .finalize(surgeist_text::SpacingAllocations::None)
            .unwrap();
        let measure = first.measurements().min_content_contribution();
        // Letter spacing cancels every actual Ahem advance10. The ordinary
        // retained NBSP additionally receives word spacing−10; it is neither
        // a residual DI nor a collapsible trailing SPACE. The genuine first
        // refinement therefore occupies[−10,0], while the source word stays0.
        let glyphs: Vec<_> = first
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(glyphs.iter().all(|(_, advance)| *advance == 10.0));
        // Retained Ahem's cmap maps U+00A0 to153; hmtx gives1000/UPEM1000.
        assert_eq!(glyphs.last(), Some(&(153, 10.0)));
        assert_eq!(
            (
                measure.advance(),
                measure.occupied_start(),
                measure.occupied_end()
            ),
            (-10.0, -10.0, 0.0)
        );
        assert!(first.run_views().all(|run| run.face().bytes()
            == include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            && run.effective_instance().unwrap().font_size() == 10.0));
        for policy in [HyphenateLimitLast::None, HyphenateLimitLast::Always] {
            let b = batch::<S>(&p, AvailableOf::MinContent, policy);
            assert_eq!(
                b.final_entries()
                    .iter()
                    .find(|row| row.node() == 0)
                    .unwrap()
                    .output()
                    .size
                    .width
                    .to_f64(),
                0.0
            );
        }
    }
    check::<f32>();
    check::<f64>();
}
