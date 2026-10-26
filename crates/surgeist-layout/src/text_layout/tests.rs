#[path = "tests/last_tail.rs"]
mod last_tail;
#[path = "tests/last_tail_composition.rs"]
mod last_tail_composition;
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use crate::*;
use surgeist_text::{ParagraphBaseDirection, ParagraphControlKind, Source, Wrap};

#[path = "tests/auto_phrase.rs"]
mod auto_phrase;
#[path = "tests/automatic_phrase.rs"]
mod automatic_phrase;
#[path = "tests/automatic_priority.rs"]
mod automatic_priority;
#[path = "tests/before_indication_bidi.rs"]
mod before_indication_bidi;
#[path = "tests/both_sides.rs"]
mod both_sides;
#[path = "tests/boundary_owner_composition.rs"]
mod boundary_owner_composition;
#[path = "tests/break_all_hyphenation.rs"]
mod break_all_hyphenation;
#[path = "tests/canonical.rs"]
mod canonical;
#[path = "tests/computed_line_boundaries.rs"]
mod computed_line_boundaries;
#[path = "tests/conditional_overflow.rs"]
mod conditional_overflow;
#[path = "tests/float_fit.rs"]
mod float_fit;
#[path = "tests/hanging_punctuation.rs"]
mod hanging_punctuation;
#[path = "tests/indent.rs"]
mod indent;
#[path = "tests/inline_boundary_edges.rs"]
mod inline_boundary_edges;
#[path = "tests/lexical_fallback.rs"]
mod lexical_fallback;
#[path = "tests/line_box_direction.rs"]
mod line_box_direction;
#[path = "tests/line_break.rs"]
mod line_break;
#[path = "tests/line_limit_composition.rs"]
mod line_limit_composition;
#[path = "tests/line_limits.rs"]
mod line_limits;
#[path = "tests/line_policy.rs"]
mod line_policy;
#[path = "tests/logical_after.rs"]
mod logical_after;
#[path = "tests/manual_css_minima.rs"]
mod manual_css_minima;
#[path = "tests/negative_tabs.rs"]
mod negative_tabs;
#[path = "tests/scope_continuation.rs"]
mod scope_continuation;
#[path = "tests/scope_domain_binding.rs"]
mod scope_domain_binding;
mod scope_regions;
#[path = "tests/tab_stops.rs"]
mod tab_stops;
#[path = "tests/wbr.rs"]
mod wbr;
#[path = "tests/whitespace_edges.rs"]
mod whitespace_edges;
#[path = "tests/whitespace_modes.rs"]
mod whitespace_modes;
#[path = "tests/word_break_break_word.rs"]
mod word_break_break_word;
#[path = "tests/word_break_manual.rs"]
mod word_break_manual;
#[path = "tests/zone_composition.rs"]
mod zone_composition;

fn assert_native_invalidation_source_order<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(90, Source::new("X"), style.clone()),
            ParagraphPart::Atomic {
                node: 3,
                size: surgeist_text::Size::new(10.0, 10.0),
            },
            ParagraphPart::Control {
                node: 44,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let binding = paragraph.binding(70);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(70, [90, 3, 44])
        .children(90, [])
        .children(3, [])
        .children(44, [])
        .style(
            70,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(90, paragraph.text_input(90))
        .style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(
            44,
            LayoutInputOf::text_control(TextControlParticipation::new(
                binding.controls[0].occurrence.clone(),
            )),
        );
    let capture = LayoutFlowInput::try_new(
        &tree,
        70,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let cold = compute_layout_with_flow(&capture, 70, request(40.0)).unwrap();
    // Native Text and its ancestor bypass geometry caches; the independent
    // atomic remains eligible until it is an actual invalidation seed.
    assert_eq!(cold.invalidated_nodes(), &[70, 90, 44]);
    let dirty = compute_layout_with_flow_invalidated(&capture, 70, request(40.0), &[3, 3]).unwrap();
    // This is actual tree order, neither numeric order nor concatenated closure
    // order. Caller duplicate seeds do not duplicate any output subject.
    assert_eq!(dirty.invalidated_nodes(), &[70, 90, 3, 44]);
    assert_eq!(dirty.final_entries(), cold.final_entries());
}

#[test]
fn native_and_dirty_atomic_invalidation_union_keeps_nonmonotonic_source_tree_order() {
    assert_native_invalidation_source_order::<f32>();
    assert_native_invalidation_source_order::<f64>();
}

fn assert_terminal_phantom_completion_scroll<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("X\n"), ahem_style(10.0, 10.0));
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
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
    let batch = compute_layout_with_flow(&capture, 0, request(20.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    let lines = placed.lines();
    // The actual Forced line uses Ahem ascent8/descent2. Its strut-only
    // Paragraph receiver retains those nominal metrics without a CSS line box.
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].range(), surgeist_text::Range::new(0, 2));
    assert_eq!(lines[0].baseline(), 8.0);
    assert_eq!(
        lines[0].bounds(),
        surgeist_text::Rect::new(0.0, 0.0, 20.0, 10.0)
    );
    let completions = placed.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let completion = completions[0].line.finalized_line();
    assert_eq!(completion.prepared_text(), paragraph.prepared);
    assert_eq!(
        (
            completion
                .selected_line()
                .request()
                .start
                .view()
                .source_before()
                .get(),
            completion
                .selected_line()
                .request()
                .end
                .view()
                .source_after()
                .get()
        ),
        (2, 2)
    );
    assert!(!completion.has_in_flow_content());
    assert_eq!(completion.feedback().baseline, 8.0);
    assert_eq!(completion.feedback().line_height, 10.0);
    assert_eq!(completions[0].origin.y, 10.0);
    let selected = completion.selected_line();
    let request = selected.request();
    assert_eq!(request.context.end, surgeist_text::LineEnd::Paragraph);
    assert!(matches!(&request.termination,
        surgeist_text::PreparedTermination::Ordinary(offer)
            if offer.view().kind() == surgeist_text::PreparedBreakKind::ParagraphEnd));
    assert_eq!(request.start, request.end);
    assert!(request.participants.is_empty());
    assert_eq!(placed.source().text().code_units(), &[0x58, 0x0a]);
    assert_eq!(
        placed
            .glyph_runs()
            .iter()
            .map(|run| run.advance())
            .sum::<f64>(),
        10.0
    );
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));
        let range = root.scroll_geometry.unwrap().physical_range();
        assert_eq!(range.x().maximum(), S::ZERO);
        // Only the real 10px Forced line contributes to this 10px scrollport.
        // The semantic receiver adds no rendered line or scroll overflow.
        assert_eq!(range.y().maximum(), S::ZERO);
    }
}

#[test]
fn terminal_strut_only_completion_retains_metrics_without_scroll_overflow() {
    assert_terminal_phantom_completion_scroll::<f32>();
    assert_terminal_phantom_completion_scroll::<f64>();
}

fn assert_intrinsic_line_box<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("X "), ahem_style(10.0, 10.0));
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
        .input(1, paragraph.text_input(1));
    let binding = paragraph.binding(0);
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.lines().len(), 1);
    let actual = placed.lines()[0].bounds();
    // The public computation resolves the auto root to 20px before the final
    // pass. That known containing size is a genuine definite line allocation.
    assert_eq!(actual, surgeist_text::Rect::new(0.0, 0.0, 20.0, 10.0));
    let finalized = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(finalized.feedback().advance, 20.0);
    assert_eq!(finalized.feedback().trailing_whitespace, 10.0);
    assert_eq!(
        finalized.selected_line().request().context.available_inline,
        Some(20.0)
    );
    let size = Size::new(S::from_f64(20.0), S::from_f64(10.0));
    let output = super::run::layout(
        &binding,
        &Default::default(),
        &[],
        &[],
        super::run::Context {
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                Direction::Ltr,
            ),
            frame: super::run::Frame::Canonical {
                origin: Point::ZERO,
                size,
            },
            containing_size: size,
            inset: Point::ZERO,
            right_inset: S::ZERO,
            cursor_block: S::ZERO,
            axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            available: AvailableOf::MAX_CONTENT,
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
        super::run::LineEnvironment {
            checkpoint_group: || {},
            reset: |_| {},
            band: |_, _| {
                Ok(super::fit::Band {
                    start: 0.0,
                    end: 20.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &surgeist_text::LineSelectionRequest| Ok(None),
            replay: |_: &surgeist_text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
    .unwrap_or_else(|_| panic!("actual intrinsic paragraph finalizes"));
    // The same owning operation with a genuinely intrinsic request has no
    // definite allocation. Both Ahem code units advance 10px; its actual line
    // box includes the conditionally hangable space for this explicit maximum
    // contribution. Its actual Text metric frame retains the same 20px width.
    let surgeist_text::PlacementEntry::Formatted(first) = &output.paragraph.entries[0] else {
        panic!("native intrinsic line remains formatted");
    };
    assert_eq!(
        first
            .line
            .selected_line()
            .request()
            .context
            .available_inline,
        None
    );
    assert_eq!(
        output
            .line_bounds
            .iter()
            .map(|bounds| bounds.allocation)
            .collect::<Vec<_>>(),
        vec![
            ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(20.0), S::from_f64(10.0)),)
                .unwrap()
        ]
    );
    assert_eq!(
        output
            .line_bounds
            .iter()
            .map(|bounds| bounds.metrics)
            .collect::<Vec<_>>(),
        vec![
            ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(20.0), S::from_f64(10.0)))
                .unwrap()
        ]
    );
    let intrinsic = surgeist_text::Layout::from_finalized_lines(surgeist_text::PlacementRequest {
        prepared: output.paragraph.prepared,
        coverage: surgeist_text::PlacementCoverage::WholeParagraph,
        entries: output.paragraph.entries,
    })
    .unwrap();
    assert_eq!(
        intrinsic.lines()[0].bounds(),
        surgeist_text::Rect::new(0.0, 0.0, 20.0, 10.0)
    );
    assert_eq!(placed.source().text().code_units(), &[0x58, 0x20]);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, size);
        let range = root.scroll_geometry.unwrap().physical_range();
        assert_eq!(range.x().maximum(), S::ZERO);
        assert_eq!(range.y().maximum(), S::ZERO);
    }
}

#[test]
fn intrinsic_trailing_whitespace_keeps_line_box_allocation_separate_from_advance() {
    assert_intrinsic_line_box::<f32>();
    assert_intrinsic_line_box::<f64>();
}

fn assert_fractional_float_clearance<S: LayoutScalar>(explicit_clear: bool) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let style = ahem_style(5.25, 10.0);
        let mut parts = vec![ParagraphPart::text(2, Source::new("X"), style.clone())];
        if explicit_clear {
            parts.extend([
                ParagraphPart::Control {
                    node: 3,
                    kind: ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(4, Source::new("X"), style.clone()),
            ]);
        }
        let paragraph = PreparedParagraph::new(
            parts,
            style,
            if direction == Direction::Ltr {
                ParagraphBaseDirection::LeftToRight
            } else {
                ParagraphBaseDirection::RightToLeft
            },
        );
        let mut children = vec![1, 2];
        if explicit_clear {
            children.extend([3, 4]);
        }
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, children)
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(20.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    direction,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(if explicit_clear { 3.0 } else { 30.5 })),
                        PreferredSizeOf::px(S::from_f64(if explicit_clear { 18.25 } else { 8.25 })),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(S::from_f64(0.25)),
                        bottom: LengthAutoOf::px(S::from_f64(0.25)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .input(2, paragraph.text_input(2));
        if explicit_clear {
            tree = tree
                .children(3, [])
                .children(4, [])
                .input(
                    3,
                    LayoutInputOf::line_break(
                        LineBreakInputOf::new()
                            .with_direction(direction)
                            .with_clear(Clear::Both),
                    ),
                )
                .input(4, paragraph.text_input(4));
        }
        let mut binding = paragraph.binding(0);
        if explicit_clear {
            binding.nodes = vec![2, 3, 4];
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
        let batch = compute_layout_with_flow(&capture, 0, request(20.0)).unwrap();
        let placed = batch.placed_text()[0].layout();
        let lines = placed.lines();
        assert_eq!(lines.len(), if explicit_clear { 2 } else { 1 });
        let target = &lines[usize::from(explicit_clear)];
        // Ahem 5.25 has before-baseline 4.2 + (10 - 5.25)/2 = 6.575.
        // The actual canonical float margin box ends at 9.25 or 19.25.
        // The previous grid baseline would still intersect it; the selected
        // baseline is the earliest permitted point on the same pixel grid.
        let expected_baseline = if explicit_clear { 26.0 } else { 16.0 };
        assert_eq!(target.baseline(), expected_baseline);
        let node = if explicit_clear { 4 } else { 2 };
        let finalized = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(node))
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        let before = finalized.feedback().baseline;
        assert!((before - 6.575).abs() < 0.000_001);
        let float = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 1)
            .unwrap()
            .output();
        let margin_end =
            float.location.y.to_f64() + float.size.height.to_f64() + float.margin.bottom.to_f64();
        assert!(expected_baseline - 1.0 - before < margin_end);
        assert!(expected_baseline - before >= margin_end);
        assert_eq!(target.bounds().origin.y, expected_baseline - before);
        assert_eq!(target.bounds().origin.x, 0.0);
        assert_eq!(target.bounds().size.width, 20.0);
        assert_eq!(
            placed
                .run_views()
                .filter(|run| run.glyphs().next().is_some())
                .count(),
            if explicit_clear { 2 } else { 1 }
        );
    }
}

#[test]
fn fractional_float_transition_selects_the_first_feasible_canonical_baseline() {
    assert_fractional_float_clearance::<f32>(false);
    assert_fractional_float_clearance::<f64>(false);
}

#[test]
fn explicit_clear_retains_its_actual_minimum_through_canonical_line_placement() {
    assert_fractional_float_clearance::<f32>(true);
    assert_fractional_float_clearance::<f64>(true);
}

fn assert_no_clear_preserves_fractional_line_position<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("X\nX\nX"), ahem_style(16.0, 20.4));
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
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
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88, 10, 88, 10, 88]);
    let lines = placed.lines();
    assert_eq!(lines.len(), 3);
    // Ahem's ascent 12.8 plus half-leading 2.2 gives a 15px baseline.
    // Logical starts remain 0, 20.4, and 40.8: the third natural baseline
    // is 55.8 and legitimately rounds to 56 without any clearance minimum.
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(paragraph.origin(1))
        );
        let feedback = fragment.finalized_line().feedback();
        assert!((feedback.baseline - 15.0).abs() < 1e-6);
        assert!((feedback.line_height - 20.4).abs() < 1e-6);
    }
    for (line, top, baseline) in [
        (&lines[0], 0.0, 15.0),
        (&lines[1], 20.0, 35.0),
        (&lines[2], 41.0, 56.0),
    ] {
        assert_eq!(line.baseline(), baseline);
        assert!((line.bounds().origin.y - top).abs() < 0.000_001);
        assert!((line.bounds().size.height - 20.4).abs() < 0.000_001);
    }
    assert!((lines[2].bounds().origin.y + 20.4 - 61.4).abs() < 1e-6);
    let natural = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    let tolerance = S::EPSILON.to_f64() * 61.4 * 8.0;
    assert!((natural.size.height.to_f64() - 61.2).abs() <= tolerance);
    let natural_scroll = natural.scroll_geometry.unwrap();
    let overflow = natural_scroll.scrollable_overflow();
    assert!(((overflow.origin().y + overflow.size().height).to_f64() - 61.4).abs() <= tolerance);
    // Visible overflow retains the actual overflowing metric bounds while
    // exposing no scroll range, independently of canonical endpoint rounding.
    assert_eq!(natural_scroll.physical_range().y().maximum(), S::ZERO);
    let canonical = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(canonical.size.height, S::from_f64(61.0));
    // The same Visible exposure rule applies to the canonical output; the
    // retained Text geometry above still has its fractional metric endpoint.
    assert_eq!(
        canonical
            .scroll_geometry
            .unwrap()
            .physical_range()
            .y()
            .maximum(),
        S::ZERO
    );
}

#[test]
fn no_clearance_does_not_turn_scalar_rounding_into_a_line_minimum() {
    assert_no_clear_preserves_fractional_line_position::<f32>();
    assert_no_clear_preserves_fractional_line_position::<f64>();
}

fn assert_actual_no_clear_round_trip<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("X\nX"), ahem_style(16.0, 20.2));
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
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
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88, 10, 88]);
    assert_eq!(placed.lines().len(), 2);
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(paragraph.origin(1))
        );
        let feedback = fragment.finalized_line().feedback();
        // Ascent 12.8 plus half-leading 2.1 supplies before 14.9.
        assert!((feedback.baseline - 14.9).abs() < 1e-6);
        assert!((feedback.line_height - 20.2).abs() < 1e-6);
    }
    assert!(f64::from(20.2_f32) > 20.2);
    // The actual next logical start is 20.2. Nearest(20.2+14.9)=35;
    // treating its f32 round-trip as a real minimum would instead ceil to 36.
    for (line, top, baseline) in [
        (&placed.lines()[0], 0.1, 15.0),
        (&placed.lines()[1], 20.1, 35.0),
    ] {
        assert_eq!(line.baseline(), baseline);
        assert!((line.bounds().origin.y - top).abs() < 1e-6);
    }
    let natural = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    let tolerance = S::EPSILON.to_f64() * 40.4 * 8.0;
    assert!((natural.size.height.to_f64() - 40.4).abs() <= tolerance);
    let canonical = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(canonical.size.height, S::from_f64(40.0));
}

#[test]
fn actual_no_clear_round_trip_keeps_nearest_baseline_in_both_scalar_lanes() {
    assert_actual_no_clear_round_trip::<f32>();
    assert_actual_no_clear_round_trip::<f64>();
}

fn assert_two_logical_allocations_survive_line_anchor_translation<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("XX XX"), ahem_style(10.5, 13.5));
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(35.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
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
    let batch = compute_layout_with_flow(&capture, 0, request(35.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88, 88, 32, 88, 88]);
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].range(), surgeist_text::Range::new(0, 3));
    assert_eq!(lines[1].range(), surgeist_text::Range::new(3, 5));
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(paragraph.origin(1))
        );
        let feedback = fragment.finalized_line().feedback();
        // Ahem ascent 8.4/descent 2.1 plus 1.5px half-leading gives 9.9/3.6.
        assert!((feedback.baseline - 9.9).abs() < 0.000_001);
        assert_eq!(feedback.line_height, 13.5);
    }
    let natural = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    let tolerance = S::EPSILON.to_f64() * 27.0 * 8.0;
    assert!(
        (natural.size.height.to_f64() - 27.0).abs() <= tolerance,
        "two genuine logical allocations consume 27px before box rounding, got {:?}",
        natural.size.height,
    );
    for (line, top, baseline) in [(&lines[0], 0.1, 10.0), (&lines[1], 13.1, 23.0)] {
        assert_eq!(line.baseline(), baseline);
        assert!((line.bounds().origin.y - top).abs() < 0.000_001);
        assert_eq!(line.bounds().size.height, 13.5);
    }
    assert!((lines[1].bounds().origin.y + 13.5 - 26.6).abs() < 0.000_001);
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    // The two normal-flow allocations sum to 27. Their rigid anchor offsets
    // affect the actual Text geometry, never the logical stacking distance.
    assert_eq!(root.size.height, S::from_f64(27.0));
}

#[test]
fn two_fractional_line_allocations_keep_logical_height_f32() {
    assert_two_logical_allocations_survive_line_anchor_translation::<f32>();
}

#[test]
fn two_fractional_line_allocations_keep_logical_height_f64() {
    assert_two_logical_allocations_survive_line_anchor_translation::<f64>();
}

fn assert_three_logical_allocations_survive_line_anchor_translation<S: LayoutScalar>() {
    let style = ahem_style(16.0, 20.0);
    let mut parts = Vec::new();
    for (text, atomic) in [(1, 2), (3, 4), (5, 6)] {
        parts.push(ParagraphPart::text(text, Source::new("X"), style.clone()));
        parts.push(ParagraphPart::Atomic {
            node: atomic,
            size: surgeist_text::Size::new(4.0, 16.0),
        });
    }
    let paragraph = PreparedParagraph::new(parts, style, ParagraphBaseDirection::LeftToRight);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3, 4, 5, 6])
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
        );
    for (text, atomic) in [(1, 2), (3, 4), (5, 6)] {
        tree = tree
            .children(text, [])
            .children(atomic, [])
            .input(text, paragraph.text_input(text))
            .style(
                atomic,
                NodeInputOf {
                    display: Display::InlineBlock,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(4.0)),
                        PreferredSizeOf::px(S::from_f64(16.0)),
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
    let batch = compute_layout_with_flow(&capture, 0, request(20.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88, 88, 88]);
    let lines = placed.lines();
    assert_eq!(lines.len(), 3);
    for (index, (top, baseline)) in [(0.0, 16.0), (21.0, 37.0), (42.0, 58.0)]
        .into_iter()
        .enumerate()
    {
        let line = &lines[index];
        assert_eq!(line.range(), surgeist_text::Range::new(index, index + 1));
        assert_eq!(line.baseline(), baseline);
        assert!((line.bounds().origin.y - top).abs() < 0.000_001);
        assert!((line.bounds().size.height - 21.2).abs() < 0.000_001);
    }
    assert_eq!(placed.inline_boxes().len(), 3);
    for node in [1, 3, 5] {
        let entry = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(node))
            .unwrap();
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(paragraph.origin(node))
        );
        let line = fragment.finalized_line();
        let feedback = line.feedback();
        // The actual bottom-baseline atomic supplies before 16; Ahem 16/LH 20
        // supplies after 3.2+2=5.2. Each genuine allocation is therefore 21.2.
        assert_eq!(feedback.baseline, 16.0);
        assert!((feedback.descent - 5.2).abs() < 0.000_001);
        assert!((feedback.line_height - 21.2).abs() < 0.000_001);
        assert_eq!(line.atomic_slots().len(), 1);
    }
    assert!((lines[2].bounds().origin.y + 21.2 - 63.2).abs() < 0.000_001);
    let natural = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    let tolerance = S::EPSILON.to_f64() * 63.6 * 8.0;
    assert!(
        (natural.size.height.to_f64() - 63.6).abs() <= tolerance,
        "three genuine logical allocations consume 63.6px before box rounding, got {:?}",
        natural.size.height,
    );
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    // Three allocations consume 63.6px and the ordinary box rounds to 64px;
    // the actual last Text metric box independently ends at 63.2px.
    assert_eq!(root.size.height, S::from_f64(64.0));
}

#[test]
fn three_fractional_line_allocations_keep_logical_height_f32() {
    assert_three_logical_allocations_survive_line_anchor_translation::<f32>();
}

#[test]
fn three_fractional_line_allocations_keep_logical_height_f64() {
    assert_three_logical_allocations_survive_line_anchor_translation::<f64>();
}

fn semantic_control_tree<S: LayoutScalar>(paragraph: &PreparedParagraph) -> PublicLayoutTreeOf<S> {
    let binding = paragraph.binding(0);
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, binding.nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    for node in &binding.nodes {
        tree = tree.children(*node, []);
        if let Some(control) = binding.controls.iter().find(|row| row.node == *node) {
            tree = tree.input(
                *node,
                LayoutInputOf::text_control(TextControlParticipation::new(
                    control.occurrence.clone(),
                )),
            );
        } else {
            tree = tree.input(*node, paragraph.text_input(*node));
        }
    }
    tree
}

fn request<S: LayoutScalar>(width: f64) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(width)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}

fn assert_optional_control<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X"), style.clone()),
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            },
            ParagraphPart::text(3, Source::new("XX"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let tree = semantic_control_tree::<S>(&paragraph);
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
    let batch = compute_layout_with_flow(&capture, 0, request(20.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].baseline(), 8.0);
    assert_eq!(lines[1].baseline(), 18.0);
    let control = paragraph.binding(0).controls[0].occurrence.clone();
    let mut finalized = Vec::new();
    for entry in batch.final_inline_fragments() {
        let line = entry.fragment().finalized_fragment().finalized_line();
        if !finalized.contains(&line) {
            finalized.push(line);
        }
    }
    let allocations = finalized
        .iter()
        .flat_map(|line| line.selected_line().request().controls.clone())
        .collect::<Vec<_>>();
    assert_eq!(allocations.len(), 1);
    assert_eq!(allocations[0].occurrence, control);
    assert!(allocations[0].strut.is_none());
}

#[test]
fn source_less_optional_control_selects_actual_break_without_a_fake_strut() {
    assert_optional_control::<f32>();
    assert_optional_control::<f64>();
}

fn assert_directional_controls<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::Control {
                node: 1,
                kind: ParagraphControlKind::RightToLeftOverride,
            },
            ParagraphPart::text(2, Source::new("AB"), style.clone()),
            ParagraphPart::Control {
                node: 3,
                kind: ParagraphControlKind::PopDirectionalFormat,
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let tree = semantic_control_tree::<S>(&paragraph);
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
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[65, 66]);
    assert_eq!(placed.lines().len(), 1);
    let line = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert!(
        line.selected_line()
            .request()
            .controls
            .iter()
            .all(|row| row.strut.is_none())
    );
    assert_eq!(line.control_slots().count(), 2);
    let clusters = placed.clusters();
    let first = clusters
        .iter()
        .find(|cluster| cluster.range().start == 0)
        .unwrap();
    let second = clusters
        .iter()
        .find(|cluster| cluster.range().start == 1)
        .unwrap();
    assert!(first.bounds().origin.x > second.bounds().origin.x);
}

#[test]
fn source_less_directional_controls_retain_original_source_and_actual_bidi_order() {
    assert_directional_controls::<f32>();
    assert_directional_controls::<f64>();
}

fn assert_discretionary_alternatives<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::with_replacements(
        vec![
            ParagraphPart::text(1, Source::new("XX"), style.clone()),
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(style.clone()),
                },
            },
            ParagraphPart::text(3, Source::new("XXXX"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
        vec![
            surgeist_text::ParagraphReplacement::resolved_native_logical_before(
                surgeist_text::ParagraphReplacementDefinition::Control {
                    control: 0,
                    origin: None,
                },
                2,
                vec!["XX".into(), "X".into()],
            )
            .unwrap(),
        ],
    );
    let opportunity = paragraph
        .prepared
        .opportunity_views()
        .find(|view| view.kind() == surgeist_text::PreparedBreakKind::Discretionary)
        .unwrap();
    let alternatives = opportunity
        .alternatives()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 2);
    let tree = semantic_control_tree::<S>(&paragraph);
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
    let batch = compute_layout_with_flow(&capture, 0, request(30.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88; 6]);
    assert_eq!(placed.lines().len(), 2);
    let replacement = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| {
            matches!(
                entry.fragment().finalized_fragment().view().owner(),
                surgeist_text::FinalizedFragmentOwner::Alternative(_)
            )
        })
        .unwrap();
    assert_eq!(replacement.owner(), &crate::InlineFragmentOwner::Node(2));
    let fragment = replacement.fragment().finalized_fragment();
    assert_eq!(
        fragment.view().owner(),
        surgeist_text::FinalizedFragmentOwner::Alternative(alternatives[1].clone())
    );
    assert!(fragment.view().origin().is_none());
    assert_eq!(
        replacement.fragment().rect().unwrap().size().width,
        S::from_f64(10.0)
    );
    let line = fragment.finalized_line();
    assert_eq!(line.feedback().advance, 30.0);
    assert_eq!(
        line.selected_line().request().alternative.as_ref(),
        Some(&alternatives[1])
    );
    assert!(line.selected_line().request().controls[0].strut.is_none());
}

#[test]
fn source_less_discretionary_tries_all_actual_alternatives_and_preserves_original_source() {
    assert_discretionary_alternatives::<f32>();
    assert_discretionary_alternatives::<f64>();
}

fn assert_atomic_alignment<S: LayoutScalar>() {
    for (vertical_align, baseline, height) in [
        (VerticalAlign::Baseline, 20.0, 22.0),
        (VerticalAlign::Top, 8.0, 20.0),
        (VerticalAlign::Bottom, 18.0, 20.0),
    ] {
        let style = ahem_style(10.0, 10.0);
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X"), style.clone()),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(20.0, 20.0),
                },
            ],
            style,
            ParagraphBaseDirection::LeftToRight,
        );
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1))
            .style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    vertical_align,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(20.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
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
        let batch = compute_layout_with_flow(&capture, 0, request(100.0)).unwrap();
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.lines()[0].baseline(), baseline, "{vertical_align:?}");
        assert_eq!(
            placed.lines()[0].bounds().size.height,
            height,
            "{vertical_align:?}"
        );
        let actual = batch
            .final_entries()
            .iter()
            .find(|row| row.node() == 2)
            .unwrap()
            .output();
        assert_eq!(
            actual.location,
            Point::new(S::from_f64(10.0), S::ZERO),
            "{vertical_align:?}"
        );
        assert_eq!(actual.size, Size::new(S::from_f64(20.0), S::from_f64(20.0)));
        let placed_box = placed.inline_boxes()[0];
        assert_eq!(placed_box.rect().origin.x, 10.0);
        assert_eq!(placed_box.rect().origin.y, 0.0);
        assert_eq!(placed_box.rect().size, surgeist_text::Size::new(20.0, 20.0));
    }
}

#[test]
fn actual_atomic_alignment_and_font_strut_share_one_final_metric_group() {
    assert_atomic_alignment::<f32>();
    assert_atomic_alignment::<f64>();
}

fn assert_rtl_asymmetric_insets<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::text(1, Source::new("XX"), style.clone())],
        style,
        ParagraphBaseDirection::RightToLeft,
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                padding: Edges {
                    left: LengthOf::px(S::from_f64(3.0)),
                    right: LengthOf::px(S::from_f64(7.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
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
    let batch = compute_layout_with_flow(&capture, 0, request(100.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.lines()[0].bounds().origin.x, 3.0);
    assert_eq!(placed.lines()[0].bounds().size.width, 90.0);
    assert_eq!(placed.run_views().next().unwrap().origin().x, 73.0);
    let node = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 1)
        .unwrap()
        .output();
    assert_eq!(node.location.x, S::from_f64(73.0));
    assert_eq!(node.size.width, S::from_f64(20.0));
}

#[test]
fn rtl_line_anchor_uses_actual_right_inset_and_physical_left_band() {
    assert_rtl_asymmetric_insets::<f32>();
    assert_rtl_asymmetric_insets::<f64>();
}

fn assert_rtl_atomic_control_slots<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("XX"), style.clone()),
            ParagraphPart::Atomic {
                node: 2,
                size: surgeist_text::Size::new(20.0, 20.0),
            },
            ParagraphPart::Control {
                node: 3,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            },
        ],
        style,
        ParagraphBaseDirection::RightToLeft,
    );
    let binding = paragraph.binding(0);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                padding: Edges {
                    left: LengthOf::px(S::from_f64(3.0)),
                    right: LengthOf::px(S::from_f64(7.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(
            3,
            LayoutInputOf::text_control(TextControlParticipation::new(
                binding.controls[0].occurrence.clone(),
            )),
        );
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
    let batch = compute_layout_with_flow(&capture, 0, request(100.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    let line = &placed.lines()[0];
    assert_eq!(line.bounds().origin.x, 3.0);
    assert_eq!(line.bounds().size.width, 90.0);
    let finalized = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(1))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let atomic = finalized.atomic_slots().next().unwrap();
    let surgeist_text::AtomicLineAllocation::InFlow { box_geometry, .. } = atomic.allocation else {
        panic!("actual in-flow atomic")
    };
    let expected_atomic = Point::new(
        S::from_f64(3.0 + atomic.origin.x + box_geometry.border.origin.x),
        S::from_f64(
            line.baseline()
                + (atomic.origin.y - finalized.feedback().baseline)
                + box_geometry.border.origin.y,
        ),
    );
    let node = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .unwrap()
        .output();
    assert_eq!(node.location, expected_atomic);
    let control = finalized.control_slots().next().unwrap();
    let node = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 3)
        .unwrap()
        .output();
    assert_eq!(
        node.location,
        Point::new(
            S::from_f64(3.0 + control.slot().x),
            S::from_f64(line.baseline() + (control.slot().y - finalized.feedback().baseline)),
        )
    );
}

#[test]
fn rtl_atomic_and_control_slots_share_the_actual_content_origin() {
    assert_rtl_atomic_control_slots::<f32>();
    assert_rtl_atomic_control_slots::<f64>();
}

fn assert_reused_preparation_runs<S: LayoutScalar>() {
    // The same preparation serves distinct real end roles; do not invent one
    // shared BlockEnd fact. Ordinary X does not consume the absent distinction.
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(1, Source::new("X"), style.clone())],
        style,
        ParagraphBaseDirection::LeftToRight,
        surgeist_text::ParagraphTerminalEdge::Unknown,
    );
    let first = paragraph.binding(0);
    let mut second = first.clone();
    second.nodes = vec![3];
    second.origins[0].node = 3;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(1.0)),
                    PreferredSizeOf::px(S::from_f64(5.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(
            3,
            LayoutInputOf::inline_text(TextParticipation::new(paragraph.origin(1))),
        );
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![first, second],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 2);
    let first = &batch.placed_text()[0];
    let second = &batch.placed_text()[1];
    assert_eq!(first.container(), 0);
    assert_eq!(second.container(), 0);
    assert_eq!(first.nodes(), &[1]);
    assert_eq!(second.nodes(), &[3]);
    assert_eq!(first.layout().lines()[0].baseline(), 8.0);
    assert_eq!(second.layout().lines()[0].baseline(), 23.0);
    let fragment = |node| {
        batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(node))
            .unwrap()
            .fragment()
            .finalized_fragment()
    };
    assert_eq!(fragment(1).view().origin(), fragment(3).view().origin());
    assert_eq!(fragment(1).finalized_line(), fragment(3).finalized_line());
    let position = first.layout().positions().next().unwrap();
    let error = second.layout().position_geometry(&position).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(surgeist_text::ErrorDetail::InvalidTextProtocol {
            reason: surgeist_text::ProtocolError::ForeignPlacement
        })
    ));
}

#[test]
fn distinct_runs_reusing_one_preparation_keep_exact_membership_and_placement_owners() {
    assert_reused_preparation_runs::<f32>();
    assert_reused_preparation_runs::<f64>();
}

fn assert_genuine_out_of_flow<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("A"), style.clone()),
            ParagraphPart::OutOfFlow {
                node: 2,
                size: surgeist_text::Size::new(20.0, 10.0),
            },
            ParagraphPart::text(3, Source::new("B"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .input(3, paragraph.text_input(3))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
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
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[65, 66]);
    assert_eq!(placed.lines().len(), 1);
    assert_eq!(placed.lines()[0].baseline(), 8.0);
    let node = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 2)
        .unwrap()
        .output();
    assert_eq!(node.location, Point::new(S::from_f64(10.0), S::ZERO));
    assert_eq!(node.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));
    let positioned = placed.inline_boxes()[0];
    assert_eq!(positioned.kind(), surgeist_text::InlineBoxKind::OutOfFlow);
    assert_eq!(
        positioned.rect(),
        surgeist_text::Rect::new(10.0, 0.0, 20.0, 10.0)
    );
    let line = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(line.selected_line().request().atomics.len(), 1);
    assert_eq!(
        line.selected_line().request().atomics[0].allocation,
        surgeist_text::AtomicLineAllocation::OutOfFlow
    );
}

#[test]
fn genuine_out_of_flow_occurrence_joins_actual_absolute_geometry_only_at_placement() {
    assert_genuine_out_of_flow::<f32>();
    assert_genuine_out_of_flow::<f64>();
}

#[test]
fn distinct_out_of_flow_occurrences_cannot_claim_one_absolute_node() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X"), style.clone()),
            ParagraphPart::OutOfFlow {
                node: 2,
                size: surgeist_text::Size::new(10.0, 10.0),
            },
            ParagraphPart::OutOfFlow {
                node: 3,
                size: surgeist_text::Size::new(10.0, 10.0),
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let mut tree = PublicLayoutTreeOf::<f32>::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .children(1, [])
        .input(1, paragraph.text_input(1));
    for node in [2, 3] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Block,
                position: Position::Absolute,
                ..NodeInputOf::default()
            },
        );
    }
    assert!(
        LayoutFlowInput::try_new(
            &tree,
            0,
            vec![paragraph.binding(0)],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new()
            }
        )
        .is_ok()
    );
    let mut binding = paragraph.binding(0);
    binding.atomics[1].node = 2;
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
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
            TextBindingError::AtomicCoverage
        ))
    ));
}

fn assert_conflicting_atomic_authority<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::Atomic {
            node: 1,
            size: surgeist_text::Size::new(20.0, 10.0),
        }],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
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
        .style(
            1,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    surgeist_text::BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                ..NodeInputOf::default()
            },
        );
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    ) else {
        panic!("conflicting level/break authority must refuse capture")
    };
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AtomicInlineParticipation {
            reason: AtomicInlineParticipationRoleError::UnexpectedForPreparedAtomic
        })
    ));
}

#[test]
fn prepared_atomic_refuses_duplicate_caller_level_and_break_authority() {
    assert_conflicting_atomic_authority::<f32>();
    assert_conflicting_atomic_authority::<f64>();
}

#[test]
fn semantic_control_requires_exact_preparation_and_structural_role() {
    let make = || {
        let style = ahem_style(10.0, 10.0);
        PreparedParagraph::new(
            vec![ParagraphPart::Control {
                node: 1,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            }],
            style,
            ParagraphBaseDirection::LeftToRight,
        )
    };
    let paragraph = make();
    let foreign = make();
    let binding = paragraph.binding(0);
    let wrong = foreign.binding(0).controls[0].occurrence.clone();
    let tree = PublicLayoutTreeOf::<f32>::new()
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
            LayoutInputOf::text_control(TextControlParticipation::new(wrong)),
        );
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    ) else {
        panic!("foreign issued control must refuse")
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantRole
        ))
    ));
    let tree = tree.input(1, LayoutInputOf::line_break(LineBreakInputOf::new()));
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    ) else {
        panic!("optional control is not an actual forced line break")
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantRole
        ))
    ));
}

fn assert_terminal_break<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X"), style.clone()),
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::ForcedBreak,
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let tree = semantic_control_tree::<S>(&paragraph);
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
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    let lines = placed.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].baseline(), 8.0);
    let completions = placed.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let completion = completions[0].line.finalized_line();
    assert_eq!(completion.prepared_text(), paragraph.prepared);
    assert_eq!(
        (
            completion
                .selected_line()
                .request()
                .start
                .view()
                .source_before()
                .get(),
            completion
                .selected_line()
                .request()
                .end
                .view()
                .source_after()
                .get()
        ),
        (1, 1)
    );
    assert!(!completion.has_in_flow_content());
    assert_eq!(completion.feedback().baseline, 8.0);
    assert_eq!(completion.feedback().line_height, 10.0);
    assert_eq!(completions[0].origin.y, 10.0);
    let selected = completion.selected_line();
    let request = selected.request();
    assert_eq!(request.context.end, surgeist_text::LineEnd::Paragraph);
    assert!(matches!(&request.termination,
        surgeist_text::PreparedTermination::Ordinary(offer)
            if offer.view().kind() == surgeist_text::PreparedBreakKind::ParagraphEnd));
    assert_eq!(request.start, request.end);
    assert!(request.participants.is_empty());
    let forced = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(
        forced.selected_line().request().context.end,
        surgeist_text::LineEnd::Forced
    );
    assert_eq!(request.start, forced.selected_line().request().end);
    assert_eq!(placed.source().text().code_units(), &[88]);
}

#[test]
fn terminal_forced_control_retains_nonformatted_paragraph_end_receipt() {
    assert_terminal_break::<f32>();
    assert_terminal_break::<f64>();
}

fn assert_fractional_line_allocation_scroll<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("XX XX"), ahem_style(10.5, 13.5));
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(35.0)),
                    PreferredSizeOf::px(S::from_f64(26.0)),
                ),
                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
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
    let batch = compute_layout_with_flow(&capture, 0, request(35.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88, 88, 32, 88, 88]);
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].range(), surgeist_text::Range::new(0, 3));
    assert_eq!(lines[1].range(), surgeist_text::Range::new(3, 5));
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(paragraph.origin(1))
        );
        let feedback = fragment.finalized_line().feedback();
        // Actual Ahem ascent 8.4 and descent 2.1, with 1.5 half-leading.
        assert!((feedback.baseline - 9.9).abs() < 1e-6);
        assert_eq!(feedback.line_height, 13.5);
    }
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size, Size::new(S::from_f64(35.0), S::from_f64(26.0)));
    // Both directly contained logical allocations contribute to scrollable
    // overflow: their end is 2 * 13.5 = 27, beyond the fixed 26px viewport.
    // The independently translated metric bottom below is only 26.6.
    let range = root.scroll_geometry.unwrap().physical_range();
    assert_eq!(range.x().maximum(), S::ZERO);
    assert_eq!(range.y().maximum(), S::from_f64(1.0));
    for (line, top, baseline) in [(&lines[0], 0.1, 10.0), (&lines[1], 13.1, 23.0)] {
        assert_eq!(line.baseline(), baseline);
        assert!((line.bounds().origin.y - top).abs() < 1e-6);
        assert_eq!(line.bounds().size.height, 13.5);
    }
    assert!((lines[1].bounds().origin.y + lines[1].bounds().size.height - 26.6).abs() < 1e-6);
}

#[test]
fn fractional_line_allocation_scroll_includes_logical_end_beyond_metric_bottom_f32() {
    assert_fractional_line_allocation_scroll::<f32>();
}

#[test]
fn fractional_line_allocation_scroll_includes_logical_end_beyond_metric_bottom_f64() {
    assert_fractional_line_allocation_scroll::<f64>();
}

struct SliverShapeTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    queries: std::cell::RefCell<Vec<(FloatExclusionQueryOf<S>, bool)>>,
}

impl<S: LayoutScalar> Traverse for SliverShapeTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for SliverShapeTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }

    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 2);
        // The supplied shape is a real rectangle x=0..10, y=26.75..30
        // within this float's 10x30 margin box. Empty intersections report
        // no shape intersection for that band, not a stock interval.
        let intersects =
            query.band_minimum() < S::from_f64(30.0) && S::from_f64(26.75) < query.band_maximum();
        self.queries.borrow_mut().push((query, intersects));
        Some(Ok(if intersects {
            FloatExclusionIntervalOf::try_new(query, S::ZERO, S::from_f64(10.0)).unwrap()
        } else {
            None
        }))
    }
}

fn assert_logical_allocation_float_sliver<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(1, Source::new("XX XX"), ahem_style(10.5, 13.5));
    let tree = SliverShapeTree {
        tree: PublicLayoutTreeOf::<S>::new()
            .children(0, [2, 1])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(35.0)),
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
                    float_exclusion: FloatExclusion::Shape,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(30.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1)),
        queries: std::cell::RefCell::new(Vec::new()),
    };
    let margin =
        ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(10.0), S::from_f64(30.0)))
            .unwrap();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let allocation =
        FloatExclusionQueryOf::try_new(margin, axes, S::from_f64(13.5), S::from_f64(27.0)).unwrap();
    let metrics =
        FloatExclusionQueryOf::try_new(margin, axes, S::from_f64(13.1), S::from_f64(26.6)).unwrap();
    let interval = tree
        .float_exclusion_interval(2, allocation)
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(interval.originating_query(), allocation);
    assert_eq!(
        (interval.minimum(), interval.maximum()),
        (S::ZERO, S::from_f64(10.0))
    );
    assert!(
        tree.float_exclusion_interval(2, metrics)
            .unwrap()
            .unwrap()
            .is_none()
    );
    tree.queries.borrow_mut().clear();

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
    let batch = compute_layout_with_flow(&capture, 0, request(35.0)).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[88, 88, 32, 88, 88]);
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].range(), surgeist_text::Range::new(0, 3));
    assert_eq!(lines[1].range(), surgeist_text::Range::new(3, 5));
    // Logical second allocation 13.5..27 intersects the actual sliver;
    // its independently placed metric footprint 13.1..26.6 does not.
    assert_eq!(lines[0].bounds().origin.x, 0.0);
    assert_eq!(lines[0].bounds().size.width, 35.0);
    assert_eq!(lines[1].bounds().origin.x, 10.0);
    assert_eq!(lines[1].bounds().size.width, 25.0);
    assert_eq!(lines[1].baseline(), 23.0);
    assert!((lines[1].bounds().origin.y + 13.5 - 26.6).abs() < 1e-6);
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(paragraph.origin(1))
        );
        assert_eq!(fragment.finalized_line().feedback().line_height, 13.5);
    }
    let queries = tree.queries.borrow();
    assert!(queries.iter().any(|(query, included)| *included
        && (query.band_minimum().to_f64() - 13.5).abs() < 1e-5
        && (query.band_maximum().to_f64() - 27.0).abs() < 1e-5));
    assert!(queries.iter().any(|(query, included)| !*included
        && (query.band_minimum().to_f64() - 13.1).abs() < 1e-5
        && (query.band_maximum().to_f64() - 26.6).abs() < 1e-5));
}

#[test]
fn logical_line_allocation_keeps_actual_shape_float_sliver_f32() {
    assert_logical_allocation_float_sliver::<f32>();
}

#[test]
fn logical_line_allocation_keeps_actual_shape_float_sliver_f64() {
    assert_logical_allocation_float_sliver::<f64>();
}

#[path = "tests/float_continuation.rs"]
mod float_continuation;

#[path = "tests/float_encounters.rs"]
mod float_encounters;

#[path = "tests/hyphenation_zone.rs"]
mod hyphenation_zone;

#[path = "tests/hyphenation_zone_policy.rs"]
mod hyphenation_zone_policy;

#[path = "tests/wrap_inside.rs"]
mod wrap_inside;

#[path = "tests/wrap_edges.rs"]
mod wrap_edges;

#[path = "tests/box_incidence.rs"]
mod box_incidence;

#[path = "tests/box_incidence_cases.rs"]
mod box_incidence_cases;
