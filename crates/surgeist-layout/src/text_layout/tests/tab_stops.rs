//! Literal Ahem geometry and the captured block grid, through the real adapter.
use super::*;
use surgeist_text::{TabContext, TabSize, WhiteSpace};

fn paragraph(source: &str, rtl: bool, block_rtl: bool, zero: bool) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = WhiteSpace::Preserve;
    style.tab_size = TabSize::length(if zero { 0.0 } else { 40.0 }).unwrap();
    PreparedParagraph::with_css_tabs(
        vec![ParagraphPart::text(1, Source::new(source), style.clone())],
        style,
        direction(rtl),
        direction(block_rtl),
    )
}

fn direction(rtl: bool) -> ParagraphBaseDirection {
    if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    }
}

fn block<S: LayoutScalar>(rtl: bool, indent: f64, align: TextAlign) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        direction: if rtl { Direction::Rtl } else { Direction::Ltr },
        text_align: align,
        text_indent: TextIndentOf {
            amount: LengthPercentageOf::px(S::from_f64(indent)).unwrap(),
            each_line: false,
            hanging: false,
        },
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}

fn batch<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    input: NodeInputOf<S>,
    float: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, if float { vec![2, 1] } else { vec![1] })
        .children(1, [])
        .style(0, input)
        .input(1, paragraph.text_input(1));
    if float {
        tree = tree.children(2, []).style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, request(100.0)).unwrap()
}

fn glyphs<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<(usize, u32, f64)> {
    batch.placed_text()[0]
        .layout()
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (run.line(), glyph.id(), run.origin().x + glyph.pen()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<surgeist_text::FinalizedLine> {
    let mut result = Vec::new();
    for fragment in batch.final_inline_fragments() {
        let line = fragment.fragment().finalized_fragment().finalized_line();
        if !result.contains(&line) {
            result.push(line);
        }
    }
    result
}

fn assert_indent_and_alignment<S: LayoutScalar>() {
    let prepared = paragraph("A\tB", false, false, false);
    for (indent, advance, a) in [(0.0, 50.0, 0.0), (10.0, 40.0, 10.0), (-10.0, 60.0, -10.0)] {
        let actual = batch::<S>(&prepared, block(false, indent, TextAlign::Start), false);
        assert_eq!(glyphs(&actual), [(0, 35, a), (0, 36, 40.0)]);
        let line = &lines(&actual)[0];
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(
            line.selected_line().request().context.tabs,
            TabContext::CssStops { origin: Some(0.0) }
        );
        assert_eq!(
            actual.placed_text()[0]
                .layout()
                .source()
                .text()
                .code_units(),
            [65, 9, 66]
        );
    }
    // Alignment moves the completed 50px line by 25px. It does not re-snap B.
    let centered = batch::<S>(&prepared, block(false, 0.0, TextAlign::Center), false);
    assert_eq!(glyphs(&centered), [(0, 35, 25.0), (0, 36, 65.0)]);
    assert_eq!(lines(&centered)[0].feedback().advance, 50.0);
}

#[test]
fn preserved_tabs_keep_the_block_grid_through_signed_indent_and_later_alignment() {
    assert_indent_and_alignment::<f32>();
    assert_indent_and_alignment::<f64>();
}

fn assert_directions<S: LayoutScalar>() {
    for (rtl, block_rtl, origin, advance, a, b) in [
        (false, false, 0.0, 50.0, 0.0, 40.0),
        (true, true, 0.0, 50.0, 90.0, 50.0),
        // Normal aligns the unchanged30px body result against the containing
        // line-box Start: right70 in RTL, left0 in LTR. Tab origins stay on
        // their captured block grid and retain body progression independently.
        (false, true, 100.0, 30.0, 70.0, 90.0),
        (true, false, -100.0, 30.0, 20.0, 0.0),
    ] {
        let prepared = paragraph("A\tB", rtl, block_rtl, false);
        let actual = batch::<S>(&prepared, block(block_rtl, 0.0, TextAlign::Start), false);
        let mut positions = glyphs(&actual);
        positions.sort_by_key(|row| row.1);
        assert_eq!(positions, [(0, 35, a), (0, 36, b)], "{rtl}/{block_rtl}");
        let line = &lines(&actual)[0];
        assert_eq!(line.feedback().advance, advance);
        let selected = line.selected_line();
        assert_eq!(selected.request().context.base_direction, direction(rtl));
        assert_eq!(
            selected.request().context.line_box_direction,
            if block_rtl {
                surgeist_text::LineBoxDirection::RightToLeft
            } else {
                surgeist_text::LineBoxDirection::LeftToRight
            }
        );
        assert_eq!(
            line.selected_line().request().context.tabs,
            TabContext::CssStops {
                origin: Some(origin)
            }
        );
    }
}

#[test]
fn preserved_tabs_distinguish_block_edge_direction_from_paragraph_progression() {
    assert_directions::<f32>();
    assert_directions::<f64>();
}

fn assert_float_band<S: LayoutScalar>() {
    let prepared = paragraph("A\tB\nA\tB", false, false, false);
    let actual = batch::<S>(&prepared, block(false, 0.0, TextAlign::Start), true);
    assert_eq!(
        glyphs(&actual),
        [(0, 35, 30.0), (0, 36, 80.0), (1, 35, 0.0), (1, 36, 40.0)]
    );
    let actual_lines = lines(&actual);
    assert_eq!(actual_lines.len(), 2);
    assert_eq!(
        actual_lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [60.0, 50.0]
    );
    assert_eq!(
        actual_lines
            .iter()
            .map(|line| line.selected_line().request().context.tabs.clone())
            .collect::<Vec<_>>(),
        [
            TabContext::CssStops {
                origin: Some(-30.0)
            },
            TabContext::CssStops { origin: Some(0.0) }
        ]
    );
}

#[test]
fn preserved_tabs_recompute_origin_when_the_next_line_clears_a_float() {
    assert_float_band::<f32>();
    assert_float_band::<f64>();
}

#[test]
fn css_tab_capture_rejects_a_different_actual_container_direction() {
    let prepared = paragraph("A\tB", false, true, false);
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, block(false, 0.0, TextAlign::Start))
        .input(1, prepared.text_input(1));
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![prepared.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .err()
    .unwrap();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::TabReferenceDirection
        ))
    ));
}

fn intrinsic(
    prepared: &PreparedParagraph,
    block_rtl: bool,
    available: AvailableOf<f64>,
    width: f64,
) -> Result<super::super::run::Output<u32, f64>, super::super::fit::FitError> {
    intrinsic_with_indent(
        prepared,
        block_rtl,
        available,
        width,
        crate::text_indent::ResolvedTextIndent::ZERO,
    )
}

fn intrinsic_with_indent(
    prepared: &PreparedParagraph,
    block_rtl: bool,
    available: AvailableOf<f64>,
    width: f64,
    indent: crate::text_indent::ResolvedTextIndent<f64>,
) -> Result<super::super::run::Output<u32, f64>, super::super::fit::FitError> {
    let binding = prepared.binding(0);
    super::super::run::layout(
        &binding,
        &Default::default(),
        &[],
        &[],
        super::super::run::Context {
            wrapping: None,
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                if block_rtl {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                },
            ),
            frame: super::super::run::Frame::Canonical {
                origin: Point::ZERO,
                size: Size::new(width, 10.0),
            },
            containing_size: Size::new(width, 10.0),
            inset: Point::ZERO,
            right_inset: 0.0,
            cursor_block: 0.0,
            axes: FlowAxes::new(
                WritingMode::HorizontalTb,
                if block_rtl {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                },
            ),
            available,
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: crate::text_layout::fit::TailContext::Unrestricted,
        },
        &[],
        super::super::run::LineEnvironment {
            reset: |_| {},
            band: |_, _| {
                Ok(super::super::fit::Band {
                    start: 0.0,
                    end: width,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &surgeist_text::LineSelectionRequest| Ok(None),
            replay: |_: &surgeist_text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
}

#[test]
fn intrinsic_css_tabs_use_the_logical_grid_across_block_directions() {
    for width in [100.0, 120.0] {
        check_intrinsic_block_directions(width);
    }
}

fn check_intrinsic_block_directions(width: f64) {
    for available in [AvailableOf::MAX_CONTENT, AvailableOf::MIN_CONTENT] {
        for rtl in [false, true] {
            let prepared = paragraph("A\tB", rtl, rtl, false);
            let output = intrinsic(&prepared, rtl, available, width)
                .unwrap_or_else(|_| panic!("shared block/paragraph edge establishes tab phase"));
            let zero = paragraph("A\tB", rtl, !rtl, true);
            let zero_output = intrinsic(&zero, !rtl, available, width)
                .unwrap_or_else(|_| panic!("zero tab consumes no unresolved phase"));
            // Preserve permits a break after TAB: min-content measures A+TAB
            // and B separately (40 and10); max-content measures the whole50.
            // Zero changes those advances to10+10 without erasing the break.
            let expected = if available == AvailableOf::MIN_CONTENT {
                (vec![40.0, 10.0], vec![10.0, 10.0])
            } else {
                (vec![50.0], vec![20.0])
            };
            assert_eq!(
                (
                    output
                        .paragraph
                        .lines
                        .iter()
                        .map(|line| line.line.feedback().advance)
                        .collect::<Vec<_>>(),
                    zero_output
                        .paragraph
                        .lines
                        .iter()
                        .map(|line| line.line.feedback().advance)
                        .collect::<Vec<_>>()
                ),
                expected,
                "{available:?}/{rtl}"
            );
            let line = &output.paragraph.lines[0].line;
            assert_eq!(
                line.selected_line().request().context.available_inline,
                None
            );
            assert_eq!(
                line.selected_line().request().context.tabs,
                TabContext::IntrinsicCssStops
            );

            let opposite = paragraph("A\tB", rtl, !rtl, false);
            let opposite_output =
                intrinsic(&opposite, !rtl, available, width).unwrap_or_else(|_| {
                    panic!("opposite-start intrinsic logical grid must be available")
                });
            assert_eq!(
                opposite_output
                    .paragraph
                    .lines
                    .iter()
                    .map(|line| line.line.feedback().advance)
                    .collect::<Vec<_>>(),
                expected.0,
                "opposite block edge: {available:?}/{rtl}"
            );
            // A Soft-ending preserved TAB keeps its raw30px but hangs out of
            // intrinsic contributions. Raw line width is not min-content size.
            let contribution = if available == AvailableOf::MIN_CONTENT {
                10.0
            } else {
                50.0
            };
            assert_eq!(output.report.inline_extent, contribution);
            assert_eq!(opposite_output.report.inline_extent, contribution);
            if available == AvailableOf::MIN_CONTENT {
                assert_eq!(
                    opposite_output
                        .paragraph
                        .lines
                        .iter()
                        .map(|line| line
                            .line
                            .measurements()
                            .min_content_contribution()
                            .advance())
                        .collect::<Vec<_>>(),
                    [10.0, 10.0]
                );
            }

            let output = zero_output;
            assert_eq!(
                output.paragraph.lines[0]
                    .line
                    .selected_line()
                    .request()
                    .context
                    .tabs,
                TabContext::IntrinsicCssStops
            );
        }
    }
}

#[test]
fn auto_root_intrinsic_sizing_refinalizes_tabs_against_the_real_used_width() {
    for rtl in [false, true] {
        let p = paragraph("A\tB", rtl, !rtl, false);
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    direction: if rtl { Direction::Ltr } else { Direction::Rtl },
                    ..NodeInputOf::default()
                },
            )
            .input(1, p.text_input(1));
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
        for (available, expected_width) in [
            (AvailableOf::MAX_CONTENT, 50.0),
            (AvailableOf::MIN_CONTENT, 10.0),
        ] {
            let actual = compute_layout_with_flow(
                &capture,
                0,
                LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT))
                    .unwrap(),
            )
            .unwrap();
            let root = actual
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output();
            assert_eq!(root.size.width, expected_width);
            let final_lines = lines(&actual);
            // The intrinsic whole line is50. At the actual opposite edge50
            // (or10), the whole used candidate is60, so wrapping keeps the
            // hanging first TAB in raw50 and moves B to its own line.
            assert_eq!(
                final_lines
                    .iter()
                    .map(|line| line.feedback().advance)
                    .collect::<Vec<_>>(),
                [50.0, 10.0]
            );
            for line in final_lines {
                assert_eq!(
                    line.selected_line().request().context.tabs,
                    TabContext::CssStops {
                        origin: Some(if rtl { -expected_width } else { expected_width })
                    }
                );
                assert_eq!(
                    line.selected_line().request().context.available_inline,
                    Some(expected_width)
                );
            }
            assert_eq!(
                actual.placed_text()[0]
                    .layout()
                    .source()
                    .text()
                    .code_units(),
                [65, 9, 66]
            );
        }
        // A fresh finite operation after both intrinsic modes must use the
        // real100px opposite edge: A10/TAB10/B10, regardless of prior results.
        let finite = compute_layout_with_flow(&capture, 0, request(100.0)).unwrap();
        assert_eq!(
            lines(&finite)
                .iter()
                .map(|line| line.feedback().advance)
                .collect::<Vec<_>>(),
            [30.0]
        );
        let cold = batch::<f64>(&p, block(!rtl, 0.0, TextAlign::Start), false);
        assert_eq!(glyphs(&finite), glyphs(&cold));
    }
}

#[test]
fn intrinsic_lines_reset_the_tab_pen_with_the_applicable_indent() {
    for rtl in [false, true] {
        let p = paragraph("A\tB\nA\tB", rtl, !rtl, false);
        for (each_line, hanging, expected_indents, expected_raw) in [
            (false, false, [10.0, 0.0], [40.0, 50.0]),
            (true, false, [10.0, 10.0], [40.0, 40.0]),
            (false, true, [0.0, 10.0], [50.0, 40.0]),
            (true, true, [0.0, 0.0], [50.0, 50.0]),
        ] {
            let output = intrinsic_with_indent(
                &p,
                !rtl,
                AvailableOf::MAX_CONTENT,
                120.0,
                crate::text_indent::ResolvedTextIndent {
                    amount: 10.0,
                    each_line,
                    hanging,
                },
            )
            .unwrap_or_else(|_| panic!("forced intrinsic lines must resolve their logical grid"));
            assert_eq!(
                output
                    .paragraph
                    .lines
                    .iter()
                    .map(|line| line.line.feedback().advance)
                    .collect::<Vec<_>>(),
                expected_raw
            );
            assert_eq!(
                output
                    .paragraph
                    .lines
                    .iter()
                    .map(|line| line.line.selected_line().request().context.indent)
                    .collect::<Vec<_>>(),
                expected_indents
            );
        }
        let p = paragraph("A\tB", rtl, !rtl, false);
        let output = intrinsic_with_indent(
            &p,
            !rtl,
            AvailableOf::MIN_CONTENT,
            120.0,
            crate::text_indent::ResolvedTextIndent {
                amount: 10.0,
                each_line: true,
                hanging: false,
            },
        )
        .unwrap_or_else(|_| panic!("soft intrinsic lines must resolve their logical grid"));
        assert_eq!(
            output
                .paragraph
                .lines
                .iter()
                .map(|line| line.line.feedback().advance)
                .collect::<Vec<_>>(),
            [30.0, 10.0]
        );
        assert_eq!(
            output
                .paragraph
                .lines
                .iter()
                .map(|line| line.line.selected_line().request().context.indent)
                .collect::<Vec<_>>(),
            [10.0, 0.0]
        );
    }
}

fn assert_spacing_feedback<S: LayoutScalar>() {
    let prepared = paragraph("A \tB", false, false, false);
    let mut input = block(false, 0.0, TextAlign::Justify);
    input.text_align_last = TextAlignLast::Justify;
    input.text_justification = TextJustification::Spaces;
    let actual = batch::<S>(&prepared, input, false);
    let line = &lines(&actual)[0];
    // Natural width50 leaves50 allocated to SPACE: SPACE becomes60, putting
    // the pen at70. The tab shifts10 to80, then B ends90. Allocated expansion
    // is not the actual line-width increase; no consumer may report100 here.
    assert_eq!(line.feedback().advance, 90.0);
    assert_eq!(
        glyphs(&actual).iter().find(|row| row.1 == 36),
        Some(&(0, 36, 80.0))
    );
    assert_eq!(
        line.selected_line().request().context.tabs,
        TabContext::CssStops { origin: Some(0.0) }
    );
}

#[test]
fn spacing_feedback_reports_expansion_absorbed_by_a_tab_instead_of_the_requested_budget() {
    assert_spacing_feedback::<f32>();
    assert_spacing_feedback::<f64>();
}
