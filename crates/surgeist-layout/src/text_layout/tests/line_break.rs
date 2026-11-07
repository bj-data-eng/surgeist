//! Native ordinary LineBreak cuts through capture, fit, finalization and placement.
//! Registered Ahem gives each displayed ASCII character one 10px advance.
use super::*;
use crate::test_support::readback::{self, Phase};
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn paragraph(
    source: &str,
    line_break: text::LineBreak,
    ancestor_wrap: Wrap,
    rtl: bool,
) -> PreparedParagraph {
    let child = text::Style {
        line_break,
        wrap: Wrap::Word,
        white_space: text::WhiteSpace::Preserve,
        overflow_wrap: text::OverflowWrap::Normal,
        ..ahem_style(10.0, 10.0)
    };
    let base = text::Style {
        wrap: ancestor_wrap,
        ..child.clone()
    };
    PreparedParagraph::new(
        vec![ParagraphPart::text(
            1,
            Source::identified(source, text::Id::from_u64(65052), 7),
            child,
        )],
        base,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    )
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: crate::AxisSizingRequestOf<S>,
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
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, {
        let containing_flow_axes = {
            let root_input = crate::LayoutTree::node_input((capture).tree(), 0);
            crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
        };
        let requested_axes = Size::new(
            width,
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::MaxContent,
            ),
        );
        let mut logical_requests = containing_flow_axes.logical_size(requested_axes);
        // These fixtures perform ordinary block-size layout independently of
        // the explicitly requested inline contribution operation.
        logical_requests.block = logical_requests
            .block
            .with_operation(crate::SizingOperation::Normal);
        let axes = containing_flow_axes.physical_size(logical_requests);
        // The old Native NoParent fixture explicitly supplied these percentage
        // bases. It supplied no independent ICB dimensions or ancestor scrollport.
        let percentage_parent = requested_axes.map(|axis| axis.space().definite_size());
        let fallback = crate::OrthogonalFallbackOf::new(
            crate::Size::NONE,
            crate::NearestScrollportOf::Absent,
            crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
        );
        crate::LayoutRootRequestOf::standalone(
            crate::SizingRequestOf::new(axes, fallback),
            containing_flow_axes,
            percentage_parent,
        )
    })
    .unwrap()
}

fn assert_lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    source: &[u16],
    expected: &[(text::Range, f64, text::LineEnd)],
) {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), source);
    assert_eq!(placed.lines().len(), expected.len());
    let origin = paragraph.origin(1);
    assert_eq!(origin.view().source().id(), Some(text::Id::from_u64(65052)));
    assert_eq!(origin.view().source().revision(), 7);
    assert_eq!(origin.view().source().text().code_units(), source);
    let mut finalized = vec![None; expected.len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        finalized[fragment.line_index()] = Some(line);
    }
    let mut baseline = 8.0;
    for (index, ((range, advance, end), line)) in expected.iter().zip(finalized).enumerate() {
        let line = line.expect("each nonempty line has its actual source fragment");
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(placed.lines()[index].range(), *range);
        assert_eq!(request.context.end, *end);
        assert!(request.alternative.is_none());
        assert!(request.incoming.is_none());
        assert!(line.hyphenation_continuation().is_none());
        let text::PreparedTermination::Ordinary(offer) = &request.termination else {
            panic!("native LineBreak must terminate at an ordinary issued opportunity")
        };
        assert_eq!(
            offer.view().kind(),
            match end {
                text::LineEnd::Soft => text::PreparedBreakKind::Allowed,
                text::LineEnd::Forced => text::PreparedBreakKind::Forced,
                text::LineEnd::Paragraph => text::PreparedBreakKind::ParagraphEnd,
            }
        );
        assert_eq!(line.feedback().advance, *advance);
        assert_eq!(line.measurements().fit().advance(), *advance);
        assert_eq!(line.feedback().line_height, 10.0);
        assert_eq!(placed.lines()[index].baseline(), baseline);
        baseline += 10.0;
        let mut finalized_advance = 0.0;
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            finalized_advance += run.advance();
            for glyph in run.glyphs() {
                assert_eq!(glyph.advance(), 10.0);
            }
        }
        assert_eq!(finalized_advance, *advance);
        let placed_advance: f64 = placed
            .inline_metric_facts()
            .runs()
            .iter()
            .filter(|run| run.line() == index)
            .map(|run| run.advance())
            .sum();
        assert_eq!(placed_advance, *advance);
        if *end == text::LineEnd::Soft {
            assert_eq!(request.end.view().source_before().get(), range.end);
            assert_eq!(request.end.view().source_after().get(), range.end);
        }
    }
}

#[test]
fn anywhere_has_equal_ordinary_priority_and_contributes_one_tcu_to_min_content() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            // Both internal edges have equal ordinary priority, so the later
            // fitting edge wins even when punctuation supplies an earlier cut.
            let punctuation = paragraph("a,b", text::LineBreak::Anywhere, Wrap::Word, rtl);
            let actual = compute::<S>(
                &punctuation,
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(S::from_f64(20.0)).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                rtl,
            );
            assert_lines(
                &actual,
                &punctuation,
                &[97, 44, 98],
                &[
                    (text::Range::new(0, 2), 20.0, text::LineEnd::Soft),
                    (text::Range::new(2, 3), 10.0, text::LineEnd::Paragraph),
                ],
            );
            for (line_break, minimum) in [
                (text::LineBreak::Normal, 30.0),
                (text::LineBreak::Anywhere, 10.0),
            ] {
                let word = paragraph("abc", line_break, Wrap::Word, rtl);
                // Ordinary cuts reduce min-content; emergency cuts would leave
                // this unseparated word's contribution at its full 30px.
                for (available, expected) in [
                    (
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Indefinite,
                            crate::SizingOperation::MinContent,
                        ),
                        minimum,
                    ),
                    (
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Indefinite,
                            crate::SizingOperation::MaxContent,
                        ),
                        30.0,
                    ),
                ] {
                    let intrinsic = compute::<S>(&word, available, rtl);
                    let root = readback::single_box_output(&intrinsic, 0, Phase::Unrounded);
                    assert_eq!(root.size.width, S::from_f64(expected));
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn anywhere_child_cannot_override_ancestor_nowrap_but_forced_break_remains() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let paragraph = paragraph("abc\nde", text::LineBreak::Anywhere, Wrap::None, rtl);
            let actual = compute::<S>(
                &paragraph,
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                rtl,
            );
            assert_lines(
                &actual,
                &paragraph,
                &[97, 98, 99, 10, 100, 101],
                &[
                    (text::Range::new(0, 4), 30.0, text::LineEnd::Forced),
                    (text::Range::new(4, 6), 20.0, text::LineEnd::Paragraph),
                ],
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}
