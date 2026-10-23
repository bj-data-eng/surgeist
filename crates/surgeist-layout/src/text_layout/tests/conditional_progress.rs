//! Candidate convergence, indivisible progress and actual conditional replay.
use super::*;

#[test]
fn native_unavoidable_overflow_advances_one_real_egc_after_float_fallbacks() {
    let expected = (
        5.0,
        (0..4)
            .map(|index| {
                line(
                    index,
                    index + 1,
                    10.0,
                    5.0,
                    0.0,
                    index as f64 * 10.0,
                    index != 3,
                )
            })
            .collect::<Vec<_>>(),
    );
    for overflow in [OverflowWrap::Anywhere, OverflowWrap::BreakWord] {
        let actual = (
            paragraph::<f32>("AAAA", overflow, Wrap::Word, AvailableOf::definite(5.0)),
            paragraph::<f64>("AAAA", overflow, Wrap::Word, AvailableOf::definite(5.0)),
        );
        assert_eq!(actual, (expected.clone(), expected.clone()));
    }
}

fn leading_band<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = OverflowWrap::Anywhere;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new(""), style.clone()),
            ParagraphPart::text(3, Source::new("XX"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(0);
    binding.nodes = vec![1, 2, 3];
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(12.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(8.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .input(3, paragraph.text_input(3));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(12.0)).unwrap();
    assert_eq!(
        facts(&batch, &paragraph.prepared, "XX"),
        [
            line(0, 1, 10.0, 12.0, 0.0, 4.0, true),
            line(1, 2, 10.0, 12.0, 0.0, 14.0, false),
        ]
    );
    let float = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 2)
        .unwrap()
        .output();
    assert_eq!(float.location, Point::ZERO);
}
#[test]
fn native_no_fitting_conditional_uses_the_real_float_transition_before_progress() {
    leading_band::<f32>();
    leading_band::<f64>();
}

fn forced_tie<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = OverflowWrap::Anywhere;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("A"), style.clone()),
            ParagraphPart::Control {
                node: 4,
                kind: ParagraphControlKind::ForcedBreak,
            },
            ParagraphPart::text(3, Source::new("X"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(0);
    binding.nodes = vec![1, 4, 2, 3];
    let control = binding.controls[0].occurrence.clone();
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 4, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(8.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .input(3, paragraph.text_input(3))
        .input(
            4,
            LayoutInputOf::text_control(TextControlParticipation::new(control)),
        );
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(20.0)).unwrap();
    let mut first = line(0, 1, 10.0, 20.0, 0.0, 0.0, false);
    first.end = LineEnd::Forced;
    assert_eq!(
        facts(&batch, &paragraph.prepared, "AX"),
        [first, line(1, 2, 10.0, 12.0, 8.0, 10.0, false)]
    );
    let float = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 2)
        .unwrap()
        .output();
    assert_eq!(float.location, Point::new(S::ZERO, S::from_f64(10.0)));
}
#[test]
fn native_forced_equality_excludes_the_float_while_the_next_line_replays_it() {
    forced_tie::<f32>();
    forced_tie::<f64>();
}

fn registered(
    bytes: Vec<u8>,
    source: &str,
    style: surgeist_text::Style,
    direction: ParagraphBaseDirection,
) -> TextParagraphBinding<u32> {
    let ancestor_wrap = style.wrap;
    let source = Source::new(source);
    let length = source.text().len();
    let analyzed = surgeist_text::AnalyzedSource::from_paragraph(surgeist_text::ParagraphInput {
        source: source.clone(),
        base_style: style,
        base_direction: direction,
        segment_breaks: surgeist_text::DocumentSegmentBreaks::LineFeedDefault,
        paragraph_strut: surgeist_text::ParagraphStrut::DefaultFont,
        origins: vec![surgeist_text::ParagraphOrigin {
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: surgeist_text::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, ancestor_wrap);
    let mut system = surgeist_text::System::new(surgeist_text::SystemOptions {
        font_catalogue: surgeist_text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let prepared = system
        .prepare(&analyzed, surgeist_text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![1],
        origins: vec![TextOriginBinding { origin, node: 1 }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    }
}

fn registered_batch<S: LayoutScalar>(
    binding: &TextParagraphBinding<u32>,
    width: f64,
    leading_float: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut binding = binding.clone();
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [])
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(binding.origins[0].origin.clone())),
        )
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
    if leading_float {
        binding.nodes.insert(0, 2);
        tree = tree.children(2, []).style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(8.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    tree = tree.children(0, binding.nodes.clone());
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap();
    compute_layout_with_text(&capture, 0, request(width)).unwrap()
}

fn finalized(batch: &CompletedLayoutBatchOf<u32, impl LayoutScalar>) -> Vec<FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let row = entry.fragment();
        let actual = row.finalized_fragment().finalized_line();
        if let Some(previous) = &lines[row.line_index()] {
            assert_eq!(*previous, actual);
        }
        lines[row.line_index()] = Some(actual);
    }
    lines.into_iter().map(Option::unwrap).collect()
}

fn qualify_whole(
    line: &FinalizedLine,
    bytes: &[u8],
    size: f32,
    advance: f64,
    expected: &[(Range, u32, f32)],
) {
    assert_eq!(line.feedback().advance, advance);
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), size);
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        glyphs.extend(
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    assert!(!glyphs.is_empty());
    glyphs.sort_by_key(|(range, _, _)| (range.start, range.end));
    assert_eq!(glyphs, expected);
}

fn nonmonotonic<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = OverflowWrap::Anywhere;
    let bytes = super::super::canonical::fonts::triple_ligature_ahem();
    let binding = registered(
        bytes.clone(),
        "AAAA",
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let whole =
        crate::test_support::text_fragment::fragment_for(&binding.prepared).finalized_line();
    qualify_whole(
        &whole,
        &bytes,
        10.0,
        20.0,
        &[(Range::new(0, 3), 58, 10.0), (Range::new(3, 4), 35, 10.0)],
    );
    let batch = registered_batch::<S>(&binding, 15.0, false);
    let lines = finalized(&batch);
    assert_eq!(
        lines
            .iter()
            .enumerate()
            .map(|(index, line)| (
                batch.placed_text()[0].layout().lines()[index].range(),
                line.feedback().advance
            ))
            .collect::<Vec<_>>(),
        [(Range::new(0, 3), 10.0), (Range::new(3, 4), 10.0)]
    );
    let glyphs = lines
        .iter()
        .flat_map(|line| {
            line.run_views().flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes);
                run.glyphs()
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [(Range::new(0, 3), 58, 10.0), (Range::new(3, 4), 35, 10.0)]
    );
    for line in lines {
        let feedback = line.feedback();
        assert!(feedback.advance <= 15.0);
        assert!(feedback.occupied_end - feedback.occupied_start <= 15.0);
    }
}
#[test]
fn native_furthest_conditional_fit_does_not_assume_monotonic_provider_advances() {
    nonmonotonic::<f32>();
    nonmonotonic::<f64>();
}

fn reshaped<S: LayoutScalar>() {
    let mut style = ahem_style(12.5, 16.0);
    style.letter_spacing = 5.0;
    style.overflow_wrap = OverflowWrap::Anywhere;
    let bytes = super::super::canonical::fonts::ligature_ahem();
    let binding = registered(
        bytes.clone(),
        "AA",
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let whole =
        crate::test_support::text_fragment::fragment_for(&binding.prepared).finalized_line();
    qualify_whole(&whole, &bytes, 12.5, 22.5, &[(Range::new(0, 2), 58, 12.5)]);
    let origin = binding.origins[0].origin.clone();
    let seam = surgeist_text::TextSeam::Origin {
        local: surgeist_text::SourcePosition::try_new(origin.view().source().text(), 1).unwrap(),
        origin,
    };
    assert!(whole.prefix_at(&seam).unwrap().interior_unit().is_some());
    let batch = registered_batch::<S>(&binding, 26.0, true);
    let lines = finalized(&batch);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].prefix_at(&seam).unwrap().interior_unit().is_none());
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(
            batch.placed_text()[0].layout().lines()[index].range(),
            Range::new(index, index + 1)
        );
        assert_eq!(line.feedback().advance, 17.5);
        let glyphs = line
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes);
                run.glyphs().map(|g| g.id()).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [35]);
        assert_eq!(
            line.selected_line().request().context.end,
            if index == 0 {
                LineEnd::Soft
            } else {
                LineEnd::Paragraph
            }
        );
    }
    assert_eq!(
        batch
            .final_entries()
            .iter()
            .find(|row| row.node() == 2)
            .unwrap()
            .output()
            .location,
        Point::ZERO
    );
    assert_eq!(
        batch.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| line.baseline())
            .collect::<Vec<_>>(),
        [12.0, 28.0]
    );
}
#[test]
fn native_conditional_cut_replaces_the_actual_shared_ligature_and_float_prefix_basis() {
    reshaped::<f32>();
    reshaped::<f64>();
}

fn arabic<S: LayoutScalar>() {
    let mut style = ahem_style(20.0, 20.0);
    style.overflow_wrap = OverflowWrap::Anywhere;
    let bytes = super::super::canonical::fonts::arabic_ahem();
    let binding = registered(
        bytes.clone(),
        "\u{628}\u{628}\u{628}",
        style,
        ParagraphBaseDirection::RightToLeft,
    );
    let wide = registered_batch::<S>(&binding, 60.0, false);
    let whole_lines = finalized(&wide);
    assert_eq!(whole_lines.len(), 1);
    let whole = whole_lines[0].clone();
    qualify_whole(
        &whole,
        &bytes,
        20.0,
        60.0,
        &[
            (Range::new(0, 1), 100, 20.0),
            (Range::new(1, 2), 277, 20.0),
            (Range::new(2, 3), 82, 20.0),
        ],
    );
    let batch = registered_batch::<S>(&binding, 20.0, false);
    let lines = finalized(&batch);
    assert_eq!(lines.len(), 3);
    let mut glyphs = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(
            batch.placed_text()[0].layout().lines()[index].range(),
            Range::new(index, index + 1)
        );
        assert_eq!(line.feedback().advance, 20.0);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            glyphs.extend(run.glyphs().map(|g| (g.range(), g.id())));
        }
    }
    glyphs.sort_by_key(|(r, _)| r.start);
    assert_eq!(
        glyphs,
        [
            (Range::new(0, 1), 100),
            (Range::new(1, 2), 277),
            (Range::new(2, 3), 82)
        ]
    );
}
#[test]
fn native_conditional_arabic_continuations_retain_the_original_base_joining_context() {
    arabic::<f32>();
    arabic::<f64>();
}

fn phantom_frontier<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = surgeist_text::WhiteSpace::Collapse;
    style.overflow_wrap = OverflowWrap::Anywhere;
    let paragraph = PreparedParagraph::plain(1, Source::new(" XXXX"), style);
    let first = paragraph.prepared.opportunity_views().next().unwrap();
    assert_eq!(first.kind(), surgeist_text::PreparedBreakKind::Allowed);
    assert_eq!(first.boundary().view().source_before().get(), 1);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(20.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), [32, 88, 88, 88, 88]);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| (line.range(), line.bounds(), line.baseline()))
            .collect::<Vec<_>>(),
        [
            (
                Range::new(0, 3),
                surgeist_text::Rect::new(0.0, 0.0, 20.0, 10.0),
                8.0
            ),
            (
                Range::new(3, 5),
                surgeist_text::Rect::new(0.0, 10.0, 20.0, 10.0),
                18.0
            ),
        ]
    );
    for line in finalized(&batch) {
        let feedback = line.feedback();
        assert_eq!(feedback.advance, 20.0);
        assert!(feedback.occupied_end - feedback.occupied_start <= 20.0);
    }
    let mut visible = Vec::new();
    for line in finalized(&batch) {
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            for glyph in run.glyphs() {
                if glyph.range().start >= 1 {
                    assert_eq!(glyph.advance(), 10.0);
                    visible.push(glyph.range());
                }
            }
        }
    }
    assert_eq!(
        visible,
        [
            Range::new(1, 2),
            Range::new(2, 3),
            Range::new(3, 4),
            Range::new(4, 5)
        ]
    );
}
#[test]
fn native_discarded_leading_space_does_not_hide_the_next_formatted_conditional_frontier() {
    phantom_frontier::<f32>();
    phantom_frontier::<f64>();
}

fn rejected_candidate<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.word_break = surgeist_text::WordBreak::BreakAll;
    let bytes = super::super::canonical::fonts::triple_ligature_ahem();
    let binding = registered(
        bytes.clone(),
        "AAAA",
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let batch = registered_batch::<S>(&binding, 23.0, true);
    let lines = finalized(&batch);
    assert_eq!(
        batch.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(3, 4)]
    );
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    assert_eq!(
        batch.placed_text()[0].layout().lines()[0].bounds(),
        surgeist_text::Rect::new(8.0, 0.0, 15.0, 10.0)
    );
    let float = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 2)
        .unwrap()
        .output();
    assert_eq!(float.location, Point::ZERO);
    assert_eq!(float.size, Size::new(S::from_f64(8.0), S::from_f64(4.0)));
    let actual = lines[0]
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            run.glyphs()
                .map(|g| (g.range(), g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, [(Range::new(0, 3), 58, 10.0)]);
}
#[test]
fn native_later_ordinary_fit_restores_its_own_float_receipt_after_a_rejected_longer_candidate() {
    rejected_candidate::<f32>();
    rejected_candidate::<f64>();
}

fn raw_minimum<S: LayoutScalar>(overflow: OverflowWrap) -> (f64, Vec<LineFact>) {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = overflow;
    let analyzed = surgeist_text::AnalyzedSource::new(Source::new("AAAA"), style).unwrap();
    let mut system = surgeist_text::System::new(surgeist_text::SystemOptions {
        font_catalogue: surgeist_text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, surgeist_text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let binding = TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: vec![1],
        origins: vec![TextOriginBinding {
            node: 1,
            origin: origin.clone(),
        }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    };
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(origin)),
        );
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::MIN_CONTENT,
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = compute_layout_with_text(&capture, 0, request).unwrap();
    let width = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 0)
        .unwrap()
        .output()
        .size
        .width
        .to_f64();
    (width, facts(&batch, &prepared, "AAAA"))
}
#[test]
fn raw_prepared_minimum_keeps_its_characterized_ordinary_overflow_inventory() {
    // Raw analysis already issues its characterized overflow opportunities;
    // this is an ordinary-profile control, not the native CSS min-content rule.
    for overflow in [
        OverflowWrap::Normal,
        OverflowWrap::BreakWord,
        OverflowWrap::Anywhere,
    ] {
        let actual = (raw_minimum::<f32>(overflow), raw_minimum::<f64>(overflow));
        let expected = if overflow == OverflowWrap::Normal {
            (40.0, vec![line(0, 4, 40.0, 40.0, 0.0, 0.0, false)])
        } else {
            (
                10.0,
                (0..4)
                    .map(|i| line(i, i + 1, 10.0, 10.0, 0.0, i as f64 * 10.0, i != 3))
                    .collect(),
            )
        };
        assert_eq!(actual, (expected.clone(), expected));
    }
}
