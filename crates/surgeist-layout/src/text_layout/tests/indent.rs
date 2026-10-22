//! CSS Text3 §8.1 / Text4 §9.1 over real prepared Ahem paragraphs.
use super::*;
use surgeist_text::WhiteSpace;

fn indent<S: LayoutScalar>(amount: f64, each_line: bool, hanging: bool) -> TextIndentOf<S> {
    TextIndentOf {
        amount: LengthPercentageOf::px(S::from_f64(amount)).unwrap(),
        each_line,
        hanging,
    }
}

fn text(node: u32, source: &str, rtl: bool) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = WhiteSpace::Collapse;
    style.wrap = Wrap::Word;
    PreparedParagraph::new(
        vec![ParagraphPart::text(
            node,
            Source::new(source),
            style.clone(),
        )],
        style,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    )
}

fn block<S: LayoutScalar>(width: f64, policy: TextIndentOf<S>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        text_indent: policy,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}

fn simple<S: LayoutScalar>(
    source: &str,
    width: f64,
    policy: TextIndentOf<S>,
    align: TextAlign,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let paragraph = text(1, source, rtl);
    let mut style = block(width, policy);
    style.text_align = align;
    style.direction = if rtl { Direction::Rtl } else { Direction::Ltr };
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, style)
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

fn assert_signed_and_alignment<S: LayoutScalar>() {
    assert_eq!(
        xs(&simple::<S>(
            "XX XX",
            55.0,
            indent(10.0, false, false),
            TextAlign::Start,
            false
        )),
        vec![(0, 10.0), (0, 20.0), (1, 0.0), (1, 10.0)]
    );
    assert_eq!(
        xs(&simple::<S>(
            "XX XX",
            45.0,
            indent(-10.0, false, false),
            TextAlign::Start,
            false
        )),
        vec![(0, -10.0), (0, 0.0), (0, 20.0), (0, 30.0)]
    );
    assert_eq!(
        xs(&simple::<S>(
            "XX",
            60.0,
            indent(10.0, false, false),
            TextAlign::Center,
            false
        )),
        vec![(0, 25.0), (0, 35.0)]
    );
    assert_eq!(
        xs(&simple::<S>(
            "XX",
            60.0,
            indent(10.0, false, false),
            TextAlign::Start,
            true
        )),
        vec![(0, 30.0), (0, 40.0)]
    );
    assert_eq!(
        xs(&simple::<S>(
            "X",
            20.0,
            indent(30.0, false, false),
            TextAlign::Start,
            false
        )),
        vec![(0, 30.0)]
    );
}

#[test]
fn native_indent_signed_fit_alignment_and_unavoidable_overflow_both_scalars() {
    assert_signed_and_alignment::<f32>();
    assert_signed_and_alignment::<f64>();
}

fn forced<S: LayoutScalar>(each_line: bool, hanging: bool) -> CompletedLayoutBatchOf<u32, S> {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = WhiteSpace::Collapse;
    style.wrap = Wrap::Word;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X X"), style.clone()),
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::ForcedBreak,
            },
            ParagraphPart::text(3, Source::new("X X"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let binding = paragraph.binding(0);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(0, block(20.0, indent(5.0, each_line, hanging)))
        .input(1, paragraph.text_input(1))
        .input(3, paragraph.text_input(3))
        .input(
            2,
            LayoutInputOf::text_control(TextControlParticipation::new(
                binding.controls[0].occurrence.clone(),
            )),
        );
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap();
    compute_layout_with_text(&capture, 0, request(20.0)).unwrap()
}

fn assert_forced_matrix<S: LayoutScalar>() {
    for (each, hanging, positions) in [
        (false, false, [5.0, 0.0, 0.0, 0.0]),
        (true, false, [5.0, 0.0, 5.0, 0.0]),
        (false, true, [0.0, 5.0, 5.0, 5.0]),
        (true, true, [0.0, 5.0, 0.0, 5.0]),
    ] {
        assert_eq!(
            xs(&forced::<S>(each, hanging)),
            positions.into_iter().enumerate().collect::<Vec<_>>(),
            "{each}/{hanging}"
        );
    }
    for each in [false, true] {
        let batch = simple::<S>(
            "\nX\n",
            30.0,
            indent(5.0, each, false),
            TextAlign::Start,
            false,
        );
        // This source uses Collapse, so LF becomes SPACE, never a Forced event.
        assert_eq!(xs(&batch), vec![(0, 5.0)]);
    }
}

#[test]
fn native_indent_each_line_hanging_uses_committed_forced_not_soft_or_collapsed_lf_both_scalars() {
    assert_forced_matrix::<f32>();
    assert_forced_matrix::<f64>();
}

fn assert_percentage_float<S: LayoutScalar>() {
    let paragraph = text(1, "XX", false);
    let policy = TextIndentOf {
        amount: LengthPercentageOf::from_percent_fraction(S::from_f64(0.25)).unwrap(),
        each_line: false,
        hanging: false,
    };
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [2, 1])
        .children(1, [])
        .children(2, [])
        .style(0, block(80.0, policy))
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(80.0)).unwrap();
    assert_eq!(xs(&batch), vec![(0, 50.0), (0, 60.0)]);
    assert_eq!(batch.placed_text()[0].layout().lines()[0].baseline(), 8.0);
}

#[test]
fn native_indent_percentage_uses_own_container_instead_of_shortened_float_band_both_scalars() {
    assert_percentage_float::<f32>();
    assert_percentage_float::<f64>();
}

fn anonymous<S: LayoutScalar>(
    first: bool,
    nested: bool,
    predecessor: Option<(Display, Position, Float)>,
) -> CompletedLayoutBatchOf<u32, S> {
    let paragraph = text(3, "X", false);
    let policy = indent(5.0, false, false);
    let mut children = vec![1];
    if !first {
        children.insert(0, 2);
    }
    if predecessor.is_some() {
        children.insert(0, 4);
    }
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, children)
        .children(2, [])
        .children(3, [])
        .style(0, block(40.0, policy))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(3, paragraph.text_input(3));
    let container = if nested { 5 } else { 1 };
    tree = tree
        .children(1, if nested { vec![5] } else { vec![3] })
        .style(
            1,
            NodeInputOf {
                block_generation: BlockGeneration::Anonymous,
                ..block(40.0, policy)
            },
        );
    if nested {
        tree = tree.children(5, [3]).style(
            5,
            NodeInputOf {
                block_generation: BlockGeneration::Anonymous,
                ..block(40.0, policy)
            },
        );
    }
    if let Some((display, position, float)) = predecessor {
        tree = tree.children(4, []).style(
            4,
            NodeInputOf {
                display,
                position,
                float,
                ..NodeInputOf::default()
            },
        );
    }
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(container)]).unwrap();
    compute_layout_with_text(&capture, 0, request(40.0)).unwrap()
}

fn assert_anonymous<S: LayoutScalar>() {
    for nested in [false, true] {
        assert_eq!(xs(&anonymous::<S>(true, nested, None)), vec![(0, 5.0)]);
        assert_eq!(xs(&anonymous::<S>(false, nested, None)), vec![(0, 0.0)]);
        for role in [
            (Display::None, Position::Relative, Float::None),
            (Display::Block, Position::Absolute, Float::None),
            (Display::Block, Position::Relative, Float::Left),
        ] {
            assert_eq!(
                xs(&anonymous::<S>(true, nested, Some(role))),
                vec![(0, 5.0)]
            );
        }
    }
}

#[test]
fn native_indent_anonymous_first_in_flow_chain_includes_empty_blocks_excludes_out_of_flow_both_scalars()
 {
    assert_anonymous::<f32>();
    assert_anonymous::<f64>();
}

fn assert_intrinsic<S: LayoutScalar>() {
    for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
        let paragraph = text(1, "XX", false);
        let policy = TextIndentOf {
            amount: LengthPercentageOf::from_coefficients(S::from_f64(5.0), S::from_f64(0.25))
                .unwrap(),
            each_line: false,
            hanging: false,
        };
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    text_indent: policy,
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1));
        let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
        let batch = compute_layout_with_text(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
        )
        .unwrap();
        let root = batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        // Two real 10px advances + 5px absolute indent. Intrinsic percentage is
        // zero, then the actual 25px container resolves 5 + .25*25 = 11.25.
        assert_eq!(root.size.width, S::from_f64(25.0));
        assert_eq!(xs(&batch), vec![(0, 11.25), (0, 21.25)]);
    }
}
#[test]
fn native_indent_intrinsics_include_absolute_component_and_zero_percentage_both_scalars() {
    assert_intrinsic::<f32>();
    assert_intrinsic::<f64>();
}

fn assert_atomic<S: LayoutScalar>() {
    for (amount, width, expected) in [
        (5.0, 25.0, [(5.0, 0.0), (15.0, 0.0), (0.0, 10.0)]),
        (-5.0, 25.0, [(-5.0, 0.0), (5.0, 0.0), (15.0, 0.0)]),
    ] {
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .style(0, block(width, indent(amount, false, false)));
        for node in 1..=3 {
            tree = tree.children(node, []).style(
                node,
                NodeInputOf {
                    display: Display::InlineBlock,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    atomic_inline_participation: Some(AtomicInlineParticipation::new(
                        BidiLevel::try_new(0).unwrap(),
                        InlineBreakOpportunity::Allowed,
                    )),
                    ..NodeInputOf::default()
                },
            );
        }
        let batch = compute_layout(&tree, 0, request(width)).unwrap();
        for (node, (x, y)) in (1..=3).zip(expected) {
            let output = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output();
            assert_eq!(output.location, Point::new(S::from_f64(x), S::from_f64(y)));
        }
    }
}
#[test]
fn box_only_atomic_fitter_applies_signed_indent_once_both_scalars() {
    assert_atomic::<f32>();
    assert_atomic::<f64>();
}

fn assert_retry<S: LayoutScalar>() {
    let paragraph = text(1, "XXXX", false);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [2, 1])
        .children(1, [])
        .children(2, [])
        .style(0, block(100.0, indent(10.0, false, false)))
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(80.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    let cold = compute_layout_with_text(&capture, 0, request(100.0)).unwrap();
    let repeated = compute_layout_with_text(&capture, 0, request(100.0)).unwrap();
    // A real 40px indivisible word cannot fit the 20px float band minus 10.
    // Moving below the float is a retry, not a committed line consuming indent.
    assert_eq!(xs(&cold), vec![(0, 10.0), (0, 20.0), (0, 30.0), (0, 40.0)]);
    assert_eq!(cold.placed_text()[0].layout().lines()[0].baseline(), 28.0);
    assert_eq!(cold.final_entries(), repeated.final_entries());
    assert_eq!(xs(&cold), xs(&repeated));
}
#[test]
fn native_indent_float_retry_keeps_first_line_state_and_replay_both_scalars() {
    assert_retry::<f32>();
    assert_retry::<f64>();
}

fn assert_invalid<S: LayoutScalar>() {
    let anonymous = NodeInputOf {
        block_generation: BlockGeneration::Anonymous,
        ..block(40.0, indent(5.0, false, false))
    };
    let root = PublicLayoutTreeOf::<S>::new()
        .children(0, [])
        .style(0, anonymous.clone());
    assert!(matches!(
        compute_layout(&root, 0, request(40.0)).unwrap_err().kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::AnonymousGeneratingBlock)
    ));
    for display in [
        Display::Flex,
        Display::Grid,
        Display::InlineBlock,
        Display::None,
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(0, block(40.0, indent(5.0, false, false)))
            .style(
                1,
                NodeInputOf {
                    display,
                    ..anonymous.clone()
                },
            );
        assert!(matches!(
            compute_layout(&tree, 0, request(40.0)).unwrap_err().kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AnonymousBlockRole)
        ));
    }
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, block(40.0, indent(7.0, false, false)))
        .style(1, anonymous);
    assert!(matches!(
        compute_layout(&tree, 0, request(40.0)).unwrap_err().kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AnonymousIndentMismatch)
    ));
}
#[test]
fn native_indent_anonymous_context_and_roles_refuse_instead_of_guessing_both_scalars() {
    assert_invalid::<f32>();
    assert_invalid::<f64>();
}

fn assert_administrative_groups<S: LayoutScalar>() {
    for (each, hanging, expected) in [
        (false, false, [5.0, 0.0]),
        (true, false, [5.0, 0.0]),
        (false, true, [0.0, 5.0]),
        (true, true, [0.0, 5.0]),
    ] {
        let first = text(1, "X", false);
        let second = text(3, "X", false);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .style(0, block(60.0, indent(5.0, each, hanging)))
            .input(1, first.text_input(1))
            .input(3, second.text_input(3))
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Right,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let capture =
            TextLayoutInput::try_new(&tree, 0, vec![first.binding(0), second.binding(0)]).unwrap();
        let batch = compute_layout_with_text(&capture, 0, request(60.0)).unwrap();
        let actual = batch
            .placed_text()
            .iter()
            .map(|entry| {
                let run = entry.layout().run_views().next().unwrap();
                let glyph = run.glyphs().next().unwrap();
                run.origin().x + glyph.pen() + f64::from(glyph.x())
            })
            .collect::<Vec<_>>();
        // A native paragraph boundary produced by a float-separated run is
        // neither a CSS block start nor a Forced break for each-line.
        assert_eq!(actual, expected);
    }
}
#[test]
fn native_indent_administrative_float_groups_do_not_restart_css_line_context_both_scalars() {
    assert_administrative_groups::<f32>();
    assert_administrative_groups::<f64>();
}

fn assert_empty_forced<S: LayoutScalar>() {
    for each in [false, true] {
        let style = ahem_style(10.0, 10.0);
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::Control {
                    node: 1,
                    kind: ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::Control {
                    node: 2,
                    kind: ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(3, Source::new("X"), style.clone()),
            ],
            style,
            ParagraphBaseDirection::LeftToRight,
        );
        let binding = paragraph.binding(0);
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .style(0, block(40.0, indent(5.0, each, false)))
            .input(3, paragraph.text_input(3));
        for (node, row) in (1..=2).zip(&binding.controls) {
            tree = tree.input(
                node,
                LayoutInputOf::text_control(TextControlParticipation::new(row.occurrence.clone())),
            );
        }
        let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap();
        let batch = compute_layout_with_text(&capture, 0, request(40.0)).unwrap();
        let lines = finalized_lines(&batch);
        assert_eq!(lines.len(), 3);
        let indents = lines
            .iter()
            .map(|line| line.selected_line().request().context.indent)
            .collect::<Vec<_>>();
        assert_eq!(
            indents,
            if each {
                vec![5.0, 5.0, 5.0]
            } else {
                vec![5.0, 0.0, 0.0]
            }
        );
        assert_eq!(xs(&batch), vec![(2, if each { 5.0 } else { 0.0 })]);
    }
}
#[test]
fn native_indent_empty_forced_lines_commit_real_progress_both_scalars() {
    assert_empty_forced::<f32>();
    assert_empty_forced::<f64>();
}

fn assert_justified<S: LayoutScalar>() {
    let paragraph = text(1, "X X XXXX", false);
    let mut style = block(45.0, indent(5.0, false, false));
    style.text_align = TextAlign::Justify;
    style.text_justification = TextJustification::Spaces;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, style)
        .input(1, paragraph.text_input(1));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(45.0)).unwrap();
    assert_eq!(
        xs(&batch),
        vec![
            (0, 5.0),
            (0, 35.0),
            (1, 0.0),
            (1, 10.0),
            (1, 20.0),
            (1, 30.0)
        ]
    );
    let lines = finalized_lines(&batch);
    let first = &lines[0];
    assert_eq!(first.feedback().advance, 40.0);
    assert_eq!(
        first.selected_line().request().context.available_inline,
        Some(45.0)
    );
    assert_eq!(first.selected_line().request().context.indent, 5.0);
    assert_eq!(first.spacing_slots().next().unwrap().advance, 20.0);
}
#[test]
fn native_indent_spacing_budget_uses_the_reduced_capacity_exactly_once_both_scalars() {
    assert_justified::<f32>();
    assert_justified::<f64>();
}

fn finalized_lines<S: LayoutScalar>(
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

fn assert_preserved_break_and_fraction<S: LayoutScalar>() {
    for (each, hanging, expected) in [
        (false, false, [0.375, 0.0]),
        (true, false, [0.375, 0.375]),
        (false, true, [0.0, 0.375]),
        (true, true, [0.0, 0.0]),
    ] {
        let mut source_style = ahem_style(10.0, 10.0);
        source_style.white_space = WhiteSpace::Preserve;
        source_style.wrap = Wrap::Word;
        let paragraph = PreparedParagraph::plain(1, Source::new("X\nX"), source_style);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(0, block(40.0, indent(0.375, each, hanging)))
            .input(1, paragraph.text_input(1));
        let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
        let batch = compute_layout_with_text(&capture, 0, request(40.0)).unwrap();
        assert_eq!(xs(&batch), vec![(0, expected[0]), (1, expected[1])]);
        assert_eq!(
            batch.placed_text()[0]
                .layout()
                .lines()
                .iter()
                .map(|line| line.baseline())
                .collect::<Vec<_>>(),
            vec![8.0, 18.0]
        );
    }
}
#[test]
fn native_indent_preserved_source_breaks_and_fractional_rigid_geometry_both_scalars() {
    assert_preserved_break_and_fraction::<f32>();
    assert_preserved_break_and_fraction::<f64>();
}

fn assert_own_anonymous_basis_and_principal_reset<S: LayoutScalar>() {
    let paragraph = text(3, "X", false);
    let policy = TextIndentOf {
        amount: LengthPercentageOf::from_percent_fraction(S::from_f64(0.25)).unwrap(),
        each_line: false,
        hanging: false,
    };
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [3])
        .children(3, [])
        .style(0, block(80.0, policy))
        .style(
            1,
            NodeInputOf {
                block_generation: BlockGeneration::Anonymous,
                ..block(20.0, policy)
            },
        )
        .input(3, paragraph.text_input(3));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(1)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(80.0)).unwrap();
    assert_eq!(xs(&batch), vec![(0, 5.0)]);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [2, 1])
        .children(2, [])
        .children(1, [3])
        .children(3, [])
        .style(0, block(80.0, indent(9.0, false, false)))
        .style(2, block(10.0, TextIndentOf::ZERO))
        .style(1, block(20.0, indent(5.0, false, false)))
        .input(3, paragraph.text_input(3));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(1)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(80.0)).unwrap();
    assert_eq!(
        xs(&batch),
        vec![(0, 5.0)],
        "a principal block starts its own first-formatted context"
    );
}
#[test]
fn native_indent_anonymous_resolves_own_basis_and_principal_resets_context_both_scalars() {
    assert_own_anonymous_basis_and_principal_reset::<f32>();
    assert_own_anonymous_basis_and_principal_reset::<f64>();
}

fn assert_scroll<S: LayoutScalar>() {
    for (amount, minimum, maximum) in [(-10.0, -10.0, 0.0), (30.0, 0.0, 20.0)] {
        let paragraph = text(1, "X", false);
        let mut style = block(20.0, indent(amount, false, false));
        style.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(0, style)
            .input(1, paragraph.text_input(1));
        let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
        let batch = compute_layout_with_text(&capture, 0, request(20.0)).unwrap();
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        let range = root.scroll_geometry.unwrap().physical_range();
        // CSS scroll origin clips the reachable LTR start range to zero;
        // the independent overflow rectangle still retains the signed origin.
        assert_eq!(
            root.scroll_geometry
                .unwrap()
                .scrollable_overflow()
                .origin()
                .x,
            S::from_f64(minimum)
        );
        assert_eq!(range.x().minimum(), S::ZERO);
        assert_eq!(range.x().maximum(), S::from_f64(maximum));
    }
}
#[test]
fn native_indent_signed_overflow_reaches_actual_scroll_geometry_both_scalars() {
    assert_scroll::<f32>();
    assert_scroll::<f64>();
}
