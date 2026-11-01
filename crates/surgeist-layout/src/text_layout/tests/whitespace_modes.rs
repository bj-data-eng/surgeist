//! Real Layout choices for the native CSS Text whitespace modes.
//! Ahem advances are one em; all widths below follow the authored source,
//! preserved/collapsed characters and actual legal after-separator cuts.
use super::*;
use surgeist_text::{Range, TabSize, WhiteSpace};

fn paragraph(text: &str, mode: WhiteSpace, wrap: Wrap, rtl: bool, tabs: bool) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = mode;
    style.wrap = wrap;
    style.tab_size = TabSize::length(40.0).unwrap();
    let direction = if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    let parts = vec![ParagraphPart::text(1, Source::new(text), style.clone())];
    if tabs {
        PreparedParagraph::with_css_tabs(parts, style, direction, direction)
    } else {
        PreparedParagraph::new(parts, style, direction)
    }
}

fn batch<S: LayoutScalar>(
    p: &PreparedParagraph,
    available: AvailableOf<S>,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .input(1, p.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn selected<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<surgeist_text::FinalizedLine> {
    let placed = batch.placed_text()[0].layout();
    let mut lines = vec![None; placed.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        for run in line.run_views() {
            assert_eq!(
                run.face().bytes(),
                include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
            );
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            assert!(run.glyphs().all(|glyph| glyph.advance() == 10.0));
        }
        lines[fragment.line_index()] = Some(line);
    }
    lines.into_iter().map(Option::unwrap).collect()
}

fn assert_copy(layout: &surgeist_text::Layout, length: usize, expected: &str) {
    let endpoint = |offset, affinity| {
        layout
            .positions()
            .find(|position| {
                position.source_cursor() == surgeist_text::Cursor::new(offset, affinity)
            })
            .unwrap()
    };
    let selection = layout
        .position_selection(
            &endpoint(0, surgeist_text::Affinity::After),
            &endpoint(length, surgeist_text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selection.source_text(), layout.source().text());
    assert_eq!(selection.plaintext(), &surgeist_text::Text::from(expected));
}

fn intrinsic<S: LayoutScalar>(
    p: &PreparedParagraph,
    available: AvailableOf<S>,
    rtl: bool,
) -> super::super::run::Output<u32, S> {
    let binding = p.binding(0);
    let size = Size::new(S::from_f64(100.0), S::from_f64(10.0));
    let tree = p.flat_text_tree::<S>(
        0,
        NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::HorizontalTb,
            direction: if rtl { Direction::Rtl } else { Direction::Ltr },
            line_box_direction_policy: crate::LineBoxDirectionPolicy::Normal,
            ..NodeInputOf::default()
        },
    );
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let [row] = flow.associations.runs() else {
        panic!("flat intrinsic fixture has one actual admitted native run");
    };
    let admitted = flow.associations.admitted_run(&flow.paragraphs, row);
    let binding = admitted.binding;
    super::super::run::layout(
        &admitted,
        &flow.associations,
        &[],
        &[],
        super::super::run::Context {
            scope_alignments: crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
                flow.scope_alignments.clone(),
                crate::text_layout::fragmentation::InlineBoxFamilies::default(),
            ),
            orientation: super::super::run::text_orientation(
                WritingMode::HorizontalTb,
                &binding.prepared,
            )
            .unwrap_or_else(|_| panic!("horizontal fixture must resolve its Text orientation")),
            inline_baseline_context: crate::InlineBaselineContext::Physical,
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                if rtl { Direction::Rtl } else { Direction::Ltr },
            ),
            frame: super::super::run::Frame::Canonical {
                origin: Point::ZERO,
                size,
            },
            containing_size: size,
            content_insets: crate::Edges::ZERO,
            cursor_block: S::ZERO,
            axes: FlowAxes::new(
                WritingMode::HorizontalTb,
                if rtl { Direction::Rtl } else { Direction::Ltr },
            ),
            available,
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: crate::text_layout::fit::TailContext::Unrestricted,
        },
        &[],
        super::super::run::LineEnvironment {
            checkpoint_group: || {},
            reset: |_| {},
            band: |_, _| {
                Ok(super::super::fit::Band {
                    start: 0.0,
                    end: 100.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &surgeist_text::LineSelectionRequest| Ok(None),
            replay: |_: &surgeist_text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
    .unwrap_or_else(|_| panic!("real prepared paragraph finalizes"))
}

fn split_spaces<S: LayoutScalar>() {
    for rtl in [false, true] {
        let p = paragraph("A  B", WhiteSpace::BreakSpaces, Wrap::Word, rtl, false);
        let actual = batch::<S>(&p, AvailableOf::definite(S::from_f64(20.0)), rtl);
        let placed = actual.placed_text()[0].layout();
        assert_eq!(placed.source().text().code_units(), [65, 32, 32, 66]);
        assert_copy(placed, 4, "A  B");
        assert_eq!(
            placed
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 4)]
        );
        for line in selected(&actual) {
            assert_eq!(line.feedback().advance, 20.0);
            assert_eq!(line.feedback().hanging_end, 0.0);
            assert_eq!(line.measurements().fit().advance(), 20.0);
            assert_eq!(line.measurements().alignment().advance(), 20.0);
        }
        for (available, expected) in [
            (AvailableOf::MIN_CONTENT, 20.0),
            (AvailableOf::MAX_CONTENT, 40.0),
        ] {
            let measured = intrinsic::<S>(&p, available, rtl);
            assert_eq!(measured.report.inline_extent, S::from_f64(expected));
        }
        let preserved = paragraph("A  B", WhiteSpace::Preserve, Wrap::Word, rtl, false);
        let actual = batch::<S>(&preserved, AvailableOf::definite(S::from_f64(20.0)), rtl);
        let lines = selected(&actual);
        assert_eq!(lines[0].feedback().advance, 30.0);
        assert_eq!(lines[0].measurements().fit().advance(), 10.0);
        assert_eq!(
            actual.placed_text()[0].layout().lines()[0].range(),
            Range::new(0, 3)
        );
    }
}

#[test]
fn break_spaces_changes_real_fit_choices_and_intrinsic_aggregation() {
    split_spaces::<f32>();
    split_spaces::<f64>();
}

fn forced_breaks<S: LayoutScalar>() {
    for rtl in [false, true] {
        for wrap in [Wrap::Word, Wrap::None] {
            let p = paragraph(
                "A \t\n B\n\nC",
                WhiteSpace::PreserveBreaks,
                wrap,
                rtl,
                false,
            );
            let measured = intrinsic::<S>(&p, AvailableOf::MAX_CONTENT, rtl);
            assert_eq!(measured.report.inline_extent, S::from_f64(10.0));
            assert_eq!(
                measured
                    .paragraph
                    .entries
                    .iter()
                    .map(|placed| placed.finalized_line().feedback().advance)
                    .collect::<Vec<_>>(),
                [10.0, 10.0, 0.0, 10.0]
            );
            assert_eq!(
                measured
                    .paragraph
                    .entries
                    .iter()
                    .map(|placed| placed
                        .finalized_line()
                        .selected_line()
                        .request()
                        .end
                        .view()
                        .source_after()
                        .get())
                    .collect::<Vec<_>>(),
                [4, 7, 8, 9]
            );
            let actual = batch::<S>(&p, AvailableOf::definite(S::from_f64(100.0)), rtl);
            let placed = actual.placed_text()[0].layout();
            assert_eq!(
                placed.source().text().code_units(),
                [65, 32, 9, 10, 32, 66, 10, 10, 67]
            );
            assert_copy(placed, 9, "A\nB\n\nC");
            assert_eq!(
                placed
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [
                    Range::new(0, 4),
                    Range::new(4, 7),
                    Range::new(7, 8),
                    Range::new(8, 9)
                ]
            );
        }
    }
}

#[test]
fn preserve_breaks_keeps_forced_empty_lines_with_and_without_wrapping() {
    forced_breaks::<f32>();
    forced_breaks::<f64>();
}

fn retained_overflow<S: LayoutScalar>() {
    for rtl in [false, true] {
        for wrap in [Wrap::Word, Wrap::None] {
            for width in [15.0, 40.0] {
                let p = paragraph("X ", WhiteSpace::BreakSpaces, wrap, rtl, false);
                let actual = batch::<S>(&p, AvailableOf::definite(S::from_f64(width)), rtl);
                let lines = selected(&actual);
                assert_eq!(lines.len(), 1);
                let line = &lines[0];
                assert_eq!(line.feedback().advance, 20.0);
                assert_eq!(
                    (line.feedback().hanging_start, line.feedback().hanging_end),
                    (0.0, 0.0)
                );
                assert_eq!(line.measurements().fit().advance(), 20.0);
                assert_eq!(line.measurements().alignment().advance(), 20.0);
                let root = actual
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output();
                assert_eq!(root.size.width, S::from_f64(width));
                let geometry = root.scroll_geometry.unwrap();
                assert_eq!(
                    geometry.scrollable_overflow().size().width,
                    S::from_f64(width.max(20.0))
                );
                if width < 20.0 {
                    let overflow = geometry.physical_range();
                    assert_eq!(
                        overflow.x().maximum() - overflow.x().minimum(),
                        S::from_f64(5.0)
                    );
                }
            }
        }
    }
}

#[test]
fn break_spaces_retains_real_trailing_overflow_in_both_directions() {
    retained_overflow::<f32>();
    retained_overflow::<f64>();
}

fn tab_contributions<S: LayoutScalar>() {
    for rtl in [false, true] {
        for (mode, minimum) in [
            (WhiteSpace::BreakSpaces, 40.0),
            (WhiteSpace::Preserve, 10.0),
        ] {
            let p = paragraph("A\tB", mode, Wrap::Word, rtl, true);
            let min = intrinsic::<S>(&p, AvailableOf::MIN_CONTENT, rtl);
            assert_eq!(min.report.inline_extent, S::from_f64(minimum));
            assert_eq!(
                min.paragraph
                    .entries
                    .iter()
                    .map(|placed| placed.finalized_line().feedback().advance)
                    .collect::<Vec<_>>(),
                [40.0, 10.0]
            );
            assert_eq!(
                min.paragraph.entries[0]
                    .finalized_line()
                    .measurements()
                    .min_content_contribution()
                    .advance(),
                minimum
            );
            let max = intrinsic::<S>(&p, AvailableOf::MAX_CONTENT, rtl);
            assert_eq!(max.report.inline_extent, S::from_f64(50.0));
            assert_eq!(max.paragraph.entries.len(), 1);
            assert_eq!(
                max.paragraph.entries[0].finalized_line().feedback().advance,
                50.0
            );
            assert_eq!(
                max.paragraph.entries[0]
                    .finalized_line()
                    .selected_line()
                    .request()
                    .context
                    .tabs,
                surgeist_text::TabContext::IntrinsicCssStops
            );
            assert_eq!(
                max.paragraph.entries[0]
                    .finalized_line()
                    .selected_line()
                    .request()
                    .context
                    .available_inline,
                None
            );
        }
    }
}

#[test]
fn break_spaces_counts_real_intrinsic_tabs_without_preserve_hanging_omission() {
    tab_contributions::<f32>();
    tab_contributions::<f64>();
}
