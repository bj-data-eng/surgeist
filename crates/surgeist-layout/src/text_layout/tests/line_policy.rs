//! Native normalized alignment and explicit real-U0020 expansion.
use super::*;
use surgeist_text::{SpacingSlotDisposition, SpacingSlotOwner, WhiteSpace};

fn prepare(source: &str, whitespace: WhiteSpace, rtl: bool) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = whitespace;
    style.wrap = Wrap::Word;
    PreparedParagraph::new(
        vec![ParagraphPart::text(1, Source::new(source), style.clone())],
        style,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    )
}

fn batch<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: f64,
    align: TextAlign,
    last: TextAlignLast,
    justification: TextJustification,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                text_align: align,
                text_align_last: last,
                text_justification: justification,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    compute_layout_with_text(&capture, 0, request(width)).unwrap()
}

fn xs<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<(usize, f64)> {
    let mut result = Vec::new();
    for run in batch.placed_text()[0].layout().run_views() {
        for glyph in run.glyphs().filter(|glyph| glyph.id() == 58) {
            result.push((
                run.line(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
            ));
        }
    }
    result
}

fn lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<surgeist_text::FinalizedLine> {
    let mut result = Vec::new();
    for entry in batch.final_inline_fragments() {
        let line = entry.fragment().finalized_fragment().finalized_line();
        if !result.contains(&line) {
            result.push(line);
        }
    }
    result
}

fn assert_alignments<S: LayoutScalar>() {
    for rtl in [false, true] {
        let paragraph = prepare("XX", WhiteSpace::Preserve, rtl);
        for (align, expected) in [
            (TextAlign::Start, if rtl { 30.0 } else { 0.0 }),
            (TextAlign::End, if rtl { 0.0 } else { 30.0 }),
            (TextAlign::Left, 0.0),
            (TextAlign::Right, 30.0),
            (TextAlign::Center, 15.0),
        ] {
            let batch = batch::<S>(
                &paragraph,
                50.0,
                align,
                TextAlignLast::Auto,
                TextJustification::None,
                rtl,
            );
            let positions = xs(&batch);
            assert_eq!(
                positions,
                vec![(0, expected), (0, expected + 10.0)],
                "{align:?}/{rtl}"
            );
            assert_eq!(lines(&batch)[0].feedback().advance, 20.0);
        }
        let centered = batch::<S>(
            &paragraph,
            50.0,
            TextAlign::Justify,
            TextAlignLast::Justify,
            TextJustification::Spaces,
            rtl,
        );
        assert_eq!(xs(&centered), vec![(0, 15.0), (0, 25.0)]);
        let automatic = batch::<S>(
            &paragraph,
            50.0,
            TextAlign::Justify,
            TextAlignLast::Auto,
            TextJustification::Spaces,
            rtl,
        );
        assert_eq!(xs(&automatic)[0].1, if rtl { 30.0 } else { 0.0 });
    }
}
#[test]
fn native_logical_physical_and_unexpandable_alignment_both_scalars() {
    assert_alignments::<f32>();
    assert_alignments::<f64>();
}

fn assert_soft_and_last<S: LayoutScalar>() {
    let paragraph = prepare("X X X", WhiteSpace::Collapse, false);
    for (last, last_x) in [(TextAlignLast::Auto, 0.0), (TextAlignLast::Center, 12.5)] {
        let batch = batch::<S>(
            &paragraph,
            35.0,
            TextAlign::Justify,
            last,
            TextJustification::Spaces,
            false,
        );
        assert_eq!(xs(&batch), vec![(0, 0.0), (0, 25.0), (1, last_x)]);
        let actual = lines(&batch);
        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0].feedback().advance, 35.0);
        let slots = actual[0].spacing_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].advance, 15.0);
        assert_eq!(
            slots[1].disposition,
            SpacingSlotDisposition::DiscardedAtLineEdge
        );
        assert_eq!(slots[1].advance, 0.0);
        assert_eq!(
            actual[0]
                .selected_line()
                .spacing_slots()
                .collect::<Vec<_>>(),
            slots.iter().map(|row| row.slot.clone()).collect::<Vec<_>>()
        );
        assert_eq!(actual[1].feedback().advance, 10.0);
    }
    let no_expansion = batch::<S>(
        &paragraph,
        35.0,
        TextAlign::Justify,
        TextAlignLast::Center,
        TextJustification::None,
        false,
    );
    assert_eq!(xs(&no_expansion), vec![(0, 2.5), (0, 22.5), (1, 12.5)]);
    assert_eq!(lines(&no_expansion)[0].feedback().advance, 30.0);
    assert_eq!(lines(&no_expansion)[0].spacing_slots().len(), 0);
    let forced = prepare("X X\nX", WhiteSpace::Preserve, false);
    let batch = batch::<S>(
        &forced,
        35.0,
        TextAlign::Justify,
        TextAlignLast::Center,
        TextJustification::Spaces,
        false,
    );
    assert_eq!(xs(&batch), vec![(0, 2.5), (0, 22.5), (1, 12.5)]);
}
#[test]
fn native_soft_expansion_and_actual_last_forced_alignment_both_scalars() {
    assert_soft_and_last::<f32>();
    assert_soft_and_last::<f64>();
}

fn assert_retained_and_discarded<S: LayoutScalar>() {
    for (source, x, trailing) in [(" X", 20.0, false), ("X ", 0.0, true)] {
        let paragraph = prepare(source, WhiteSpace::Preserve, false);
        let batch = batch::<S>(
            &paragraph,
            30.0,
            TextAlign::Start,
            TextAlignLast::Justify,
            TextJustification::Spaces,
            false,
        );
        assert_eq!(xs(&batch), vec![(0, x)]);
        let line = lines(&batch).remove(0);
        assert_eq!(line.feedback().advance, 30.0);
        let slot = line.spacing_slots().next().unwrap();
        assert_eq!(
            slot.disposition,
            SpacingSlotDisposition::Retained { trailing }
        );
        assert_eq!(slot.advance, 20.0);
        assert!(matches!(slot.slot.owner(), SpacingSlotOwner::Unit(_)));
    }
    let paragraph = prepare("  X  ", WhiteSpace::Collapse, false);
    let batch = batch::<S>(
        &paragraph,
        30.0,
        TextAlign::Justify,
        TextAlignLast::Justify,
        TextJustification::Spaces,
        false,
    );
    assert_eq!(xs(&batch), vec![(0, 10.0)]);
    let line = lines(&batch).remove(0);
    assert_eq!(line.feedback().advance, 10.0);
    assert!(
        line.spacing_slots().all(|row| row.advance == 0.0
            && row.disposition == SpacingSlotDisposition::DiscardedAtLineEdge)
    );
}
#[test]
fn native_preserved_edges_expand_raw_advance_while_discarded_edges_stay_zero_both_scalars() {
    assert_retained_and_discarded::<f32>();
    assert_retained_and_discarded::<f64>();
}

fn assert_block_alignment<S: LayoutScalar>() {
    for (align, x) in [(TextAlign::Center, 0.0), (TextAlign::LegacyCenter, 20.0)] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    text_align: align,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(50.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 0, request(50.0)).unwrap();
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 1)
                .unwrap()
                .output()
                .location
                .x,
            S::from_f64(x)
        );
    }
}
#[test]
fn normal_center_does_not_align_ordinary_block_children_but_legacy_center_does() {
    assert_block_alignment::<f32>();
    assert_block_alignment::<f64>();
}

fn assert_alternative_spacing<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = WhiteSpace::Collapse;
    style.wrap = Wrap::Word;
    let paragraph = PreparedParagraph::with_replacements(
        vec![ParagraphPart::text(
            1,
            Source::new("X\u{ad}XXXX"),
            style.clone(),
        )],
        style,
        ParagraphBaseDirection::LeftToRight,
        vec![surgeist_text::ParagraphReplacement {
            definition: surgeist_text::ParagraphReplacementDefinition::Source {
                origin: 0,
                source: surgeist_text::Range::new(1, 2),
            },
            insertion_anchor: 2,
            style_anchor: 0,
            text: " X ".into(),
        }],
    );
    let alternative = paragraph
        .prepared
        .opportunity_views()
        .flat_map(|view| {
            view.alternatives()
                .map(|alt| alt.handle())
                .collect::<Vec<_>>()
        })
        .next()
        .unwrap();
    let batch = batch::<S>(
        &paragraph,
        35.0,
        TextAlign::Justify,
        TextAlignLast::Auto,
        TextJustification::Spaces,
        false,
    );
    let actual = lines(&batch);
    assert_eq!(actual[0].feedback().advance, 35.0);
    assert_eq!(
        actual[0].selected_line().request().alternative.as_ref(),
        Some(&alternative)
    );
    let slots = actual[0].spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    for row in &slots {
        assert_eq!(
            row.slot.owner(),
            SpacingSlotOwner::Alternative(alternative.clone())
        );
    }
    assert_eq!(slots[0].advance, 15.0);
    assert_eq!(slots[1].advance, 0.0);
    assert_eq!(
        slots[1].disposition,
        SpacingSlotDisposition::DiscardedAtLineEdge
    );
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        &[88, 0xad, 88, 88, 88, 88]
    );
}
#[test]
fn native_replacement_expansion_retains_the_actual_alternative_slot_owner() {
    assert_alternative_spacing::<f32>();
    assert_alternative_spacing::<f64>();
}

fn assert_float_amounts<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = WhiteSpace::Collapse;
    style.wrap = Wrap::Word;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X "), style.clone()),
            ParagraphPart::Atomic {
                node: 2,
                size: surgeist_text::Size::new(10.0, 10.0),
            },
            ParagraphPart::Control {
                node: 3,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            },
            ParagraphPart::text(4, Source::new("XXXX"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let binding = paragraph.binding(0);
    let mut previous_slot = None;
    for (with_float, advance, atomic_x) in [(true, 15.0, 25.0), (false, 25.0, 35.0)] {
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(
                0,
                if with_float {
                    vec![9, 1, 2, 3, 4]
                } else {
                    vec![1, 2, 3, 4]
                },
            )
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    text_align: TextAlign::Justify,
                    text_justification: TextJustification::Spaces,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(45.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .children(1, [])
            .input(1, paragraph.text_input(1))
            .children(4, [])
            .input(4, paragraph.text_input(4))
            .children(2, [])
            .style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .children(3, [])
            .input(
                3,
                LayoutInputOf::text_control(TextControlParticipation::new(
                    binding.controls[0].occurrence.clone(),
                )),
            );
        if with_float {
            tree = tree.children(9, []).style(
                9,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
        let capture = TextLayoutInput::try_new(&tree, 0, vec![binding.clone()]).unwrap();
        let batch = compute_layout_with_text(&capture, 0, request(45.0)).unwrap();
        let first = lines(&batch).remove(0);
        let slot = first.spacing_slots().next().unwrap();
        assert_eq!(slot.advance, advance);
        assert_eq!(
            first.feedback().advance,
            if with_float { 35.0 } else { 45.0 }
        );
        assert_eq!(first.atomic_slots().next().unwrap().origin.x, atomic_x);
        if let Some(previous) = previous_slot {
            assert_ne!(slot.slot, previous, "new selection owns its own slot");
        }
        previous_slot = Some(slot.slot);
        let atom = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(atom.location.x, S::from_f64(35.0));
        assert_eq!(atom.size.width, S::from_f64(10.0));
    }
}
#[test]
fn native_float_band_changes_recompute_amounts_and_actual_atomic_placement() {
    assert_float_amounts::<f32>();
    assert_float_amounts::<f64>();
}

#[test]
fn native_fractional_expansion_keeps_three_words_inside_the_actual_f64_band() {
    // Each Ahem code point advances exactly its genuine f32 font size. All
    // five natural advances are dyadic, so only the requested expansion has a
    // non-dyadic residual. Later XXXX forces this first line to end softly.
    let widths = [4.640625_f32, 4.09375, 0.28125, 14.046875, 0.21875];
    let mut base = ahem_style(10.0, 20.0);
    base.white_space = WhiteSpace::Collapse;
    base.wrap = Wrap::Word;
    let parts = ["X", " ", "X", " ", "X", " ", "XXXX"]
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            let mut style = base.clone();
            style.size = widths.get(index).copied().unwrap_or(10.0);
            ParagraphPart::text(
                u32::try_from(index + 1).unwrap(),
                Source::new(source),
                style,
            )
        })
        .collect();
    let paragraph = PreparedParagraph::new(parts, base, ParagraphBaseDirection::LeftToRight);
    assert_eq!(
        paragraph
            .prepared
            .unit_views()
            .take(5)
            .map(|unit| unit.advance())
            .collect::<Vec<_>>(),
        widths.map(f64::from)
    );

    // Node boxes use canonical CSS-pixel rounding. The real shape provider
    // retains a fractional exclusion, making the actual fitting budget remain
    // 31-.34875 = 30.65125 through every canonical feedback pass.
    struct ShapeTree(PublicLayoutTreeOf<f64>);
    impl Traverse for ShapeTree {
        type Node = u32;
        type Scalar = f64;
        type Children<'a> = <PublicLayoutTreeOf<f64> as Traverse>::Children<'a>;
        fn children(&self, node: u32) -> Self::Children<'_> {
            Traverse::children(&self.0, node)
        }
        fn child_count(&self, node: u32) -> usize {
            self.0.child_count(node)
        }
        fn child(&self, node: u32, index: usize) -> u32 {
            self.0.child(node, index)
        }
    }
    impl LayoutTree for ShapeTree {
        type MeasureError = core::convert::Infallible;
        fn node_input(&self, node: u32) -> &NodeInputOf<f64> {
            self.0.node_input(node)
        }
        fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, f64> {
            self.0.layout_input(node)
        }
        fn float_exclusion_interval(
            &self,
            node: u32,
            query: FloatExclusionQueryOf<f64>,
        ) -> Option<Result<Option<FloatExclusionIntervalOf<f64>>, Self::MeasureError>> {
            assert_eq!(node, 9);
            Some(Ok(
                FloatExclusionIntervalOf::try_new(query, 0.0, 0.34875).unwrap()
            ))
        }
    }
    let mut tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [9, 1, 2, 3, 4, 5, 6, 7])
        .children(9, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                text_align: TextAlign::Justify,
                text_justification: TextJustification::Spaces,
                size: Size::new(PreferredSizeOf::px(31.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            9,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                float_exclusion: FloatExclusion::Shape,
                size: Size::new(PreferredSizeOf::px(1.0), PreferredSizeOf::px(100.0)),
                ..NodeInputOf::default()
            },
        );
    for node in 1..=7 {
        tree = tree
            .children(node, [])
            .input(node, paragraph.text_input(node));
    }
    let tree = ShapeTree(tree);
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(31.0)).unwrap();
    let actual = lines(&batch);
    let first = &actual[0];
    assert_eq!(
        first.selected_line().request().context.available_inline,
        Some(30.65125)
    );
    assert_eq!(
        first.selected_line().request().context.end,
        surgeist_text::LineEnd::Soft
    );
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        Source::new("X X X XXXX").text().code_units()
    );
    // Independently execute the rejected three-word candidate with the exact
    // same issued inventory and the ideal equal-share amounts. Text's actual
    // per-slot additions, not an ideal sum of allocations, own the result.
    let end = paragraph
        .prepared
        .boundary_views()
        .find(|view| view.participant_offset() == 6)
        .unwrap()
        .handle();
    let termination = paragraph
        .prepared
        .opportunity_views()
        .find(|view| {
            view.boundary() == end && view.kind() == surgeist_text::PreparedBreakKind::Allowed
        })
        .unwrap()
        .handle();
    let mut proposed = first.selected_line().request().clone();
    proposed.end = end;
    proposed.termination = surgeist_text::PreparedTermination::Ordinary(termination);
    proposed.participants = paragraph.prepared.participants().take(6).collect();
    let selected = paragraph.prepared.select(proposed).unwrap();
    let natural = selected
        .finalize(surgeist_text::SpacingAllocations::None)
        .unwrap();
    assert_eq!(natural.feedback().advance, 23.28125);
    let extra = 30.65125 - natural.feedback().advance;
    let expanded = selected
        .finalize(surgeist_text::SpacingAllocations::Complete(
            selected
                .spacing_slots()
                .enumerate()
                .map(|(index, slot)| surgeist_text::SpacingAllocation {
                    slot,
                    amount: match index {
                        0 => extra / 2.0,
                        1 => extra - extra / 2.0,
                        _ => 0.0,
                    },
                })
                .collect(),
        ))
        .unwrap();
    assert_eq!(expanded.feedback().advance, 30.651250000000005);
    assert!(expanded.feedback().advance > 30.65125);
    assert_eq!(
        batch.placed_text()[0].layout().lines()[0].range(),
        surgeist_text::Range::new(0, 6),
        "all three natural words fit the genuine band before supported expansion"
    );
    assert_eq!(xs(&batch).iter().filter(|(line, _)| *line == 0).count(), 3);
    assert!(
        first.feedback().advance <= 30.65125,
        "actual logical advance must stay inside the budget: {:?}",
        first.feedback()
    );
}
