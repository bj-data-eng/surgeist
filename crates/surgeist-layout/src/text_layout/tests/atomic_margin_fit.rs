//! Signed CSS atomic margin-box fitting retains real occupied geometry.
//! CSS Text requires legal breaks at actual box margin edges.

use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn paragraph() -> PreparedParagraph {
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::Word,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        ..ahem_style(10.0, 10.0)
    };
    PreparedParagraph::new(
        vec![
            ParagraphPart::text(
                1,
                Source::identified("A", text::Id::from_u64(72_001), 1),
                style.clone(),
            ),
            ParagraphPart::Atomic {
                node: 2,
                size: text::Size::new(10.0, 10.0),
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn actual_source_premises(paragraph: &PreparedParagraph) {
    let prepared = &paragraph.prepared;
    let origin = paragraph.origin(1);
    assert_eq!(
        origin.view().source().id(),
        Some(text::Id::from_u64(72_001))
    );
    assert_eq!(origin.view().source().text(), &text::Text::from("A"));
    assert_eq!(origin.view().local_range(), text::Range::new(0, 1));
    assert_eq!(origin.view().aggregate_range(), text::Range::new(0, 1));
    let views = prepared.atomic_views().collect::<Vec<_>>();
    let [atomic] = views.as_slice() else {
        panic!("one genuine issued in-flow atomic");
    };
    assert_eq!(atomic.original_box().kind(), text::InlineBoxKind::InFlow);
    assert_eq!(atomic.original_box().size(), text::Size::new(10.0, 10.0));
    assert_eq!(atomic.source_position().get(), 1);
    let binding = paragraph.binding(0);
    assert_eq!(binding.atomics.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(binding.atomics[0].occurrence, atomic.handle());
    let participants = prepared.participants().collect::<Vec<_>>();
    let [
        text::PreparedParticipant::Text(unit),
        text::PreparedParticipant::Atomic(actual),
    ] = participants.as_slice()
    else {
        panic!("actual source Text unit followed by its issued atomic");
    };
    assert_eq!(unit.view().source_range(), text::Range::new(0, 1));
    assert_eq!(*actual, atomic.handle());
    // Do not infer the break from equal source offset1: ParagraphEnd AFTER the
    // box has that SAME offset. Observe the actual mixed participant cut1.
    let before = prepared
        .opportunity_views()
        .filter(|offer| offer.boundary().view().participant_offset() == 1)
        .collect::<Vec<_>>();
    let [offer] = before.as_slice() else {
        panic!("one real before-atomic opportunity");
    };
    assert_eq!(offer.kind(), text::PreparedBreakKind::Allowed);
    assert_eq!(offer.boundary().view().source_before().get(), 1);
    assert_eq!(offer.boundary().view().source_after().get(), 1);
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: f64,
    start_margin: f64,
    end_margin: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let n = S::from_f64;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::px(n(width)), PreferredSizeOf::AUTO),
                text_align: TextAlign::Start,
                text_wrap_style: TextWrapStyle::Auto,
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                margin: Edges {
                    left: LengthAutoOf::px(n(start_margin)),
                    right: LengthAutoOf::px(n(end_margin)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );
    // Atomic participation comes from the genuine issued binding, not a
    // constructed level/opportunity or a numeric stand-in for source membership.
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
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(n(width)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn whole_line<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> text::FinalizedLine {
    batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().line_index() == 0)
        .expect("real source A has its genuine line0 fragment")
        .fragment()
        .finalized_fragment()
        .finalized_line()
}

fn one_line<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    width: f64,
    end_margin: f64,
) {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(
        placed.analyzed_source().identity(),
        paragraph.prepared.analyzed_source().identity()
    );
    assert_eq!(placed.source().text(), &text::Text::from("A"));
    assert_eq!(
        placed.lines().len(),
        1,
        "A10 plus actual atomic margin-box(10-5)=5 must fit band15 without borrowing border occupancy as flow"
    );
    let line = whole_line(batch);
    assert_eq!(line.prepared_text(), paragraph.prepared);
    let selected = line.selected_line();
    let request = selected.request();
    let segment = paragraph.prepared.source_segments().next().unwrap();
    assert_eq!(request.start, segment.start());
    assert_eq!(request.end, segment.end());
    assert_eq!(
        request.participants,
        paragraph.prepared.participants().collect::<Vec<_>>()
    );
    assert_eq!(request.context.available_inline, Some(width));
    assert!(request.alternative.is_none() && request.incoming.is_none());
    let expected_flow = 20.0 + end_margin;
    assert_eq!(line.feedback().advance, expected_flow);
    assert_eq!(
        (line.feedback().occupied_start, line.feedback().occupied_end),
        (0.0, 20.0)
    );
    assert_eq!(line.measurements().fit().advance(), expected_flow);
    assert_eq!(
        (
            line.measurements().fit().occupied_start(),
            line.measurements().fit().occupied_end()
        ),
        (0.0, 20.0)
    );
    // Actual empty InlineBlock baseline is its bottom10. With Ahem8/+2 this
    // means line baseline10/height12, atomic top0 and text font-metric top2.
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (10.0, 12.0)
    );
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            let instance = run.effective_instance().unwrap();
            let metrics = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(text::Range::new(0, 1), 35, 0.0, 10.0)]);
    let slots = line.atomic_slots().collect::<Vec<_>>();
    let [slot] = slots.as_slice() else {
        panic!("same line has its actual atomic slot");
    };
    assert_eq!(
        slot.occurrence,
        paragraph.prepared.atomic_views().next().unwrap().handle()
    );
    let text::AtomicLineAllocation::InFlow {
        flow, box_geometry, ..
    } = slot.allocation
    else {
        panic!("actual in-flow allocation");
    };
    assert_eq!(flow.advance, 10.0 + end_margin);
    assert_eq!((flow.occupied_start, flow.occupied_end), (0.0, 10.0));
    assert_eq!((flow.before_baseline, flow.after_baseline), (10.0, 0.0));
    assert_eq!(slot.origin.x, 10.0);
    assert_eq!(slot.origin.x + box_geometry.border.origin.x, 10.0);
    assert_eq!(slot.origin.y + box_geometry.border.origin.y, 0.0);
    assert_eq!(box_geometry.border.size, text::Size::new(10.0, 10.0));
    assert_eq!(placed.inline_boxes().len(), 1);
    assert_eq!(
        placed.inline_boxes()[0].rect(),
        text::Rect::new(10.0, 0.0, 10.0, 10.0)
    );
    // Font-metric source geometry is a separate query from the line/cluster
    // band. Baseline10 minus Ahem ascent8 gives top2, with font height8+2.
    let range = placed
        .source_range_fragments(&paragraph.origin(1), text::Range::new(0, 1))
        .unwrap();
    let font_rects = range
        .units()
        .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(font_rects, [text::Rect::new(0.0, 2.0, 10.0, 10.0)]);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, Size::new(S::from_f64(width), S::from_f64(12.0)));
        let atomic = entries
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(atomic.location, Point::new(S::from_f64(10.0), S::ZERO));
        assert_eq!(atomic.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
        assert_eq!(
            atomic.margin,
            Edges {
                right: S::from_f64(end_margin),
                ..Edges::ZERO
            }
        );
    }
}

fn atomic_margin_lane<S: LayoutScalar>() {
    let paragraph = paragraph();
    actual_source_premises(&paragraph);
    // Real source A plus border10 needs20
    // with zero margin; the actual negative-margin full candidate at band20
    // publishes flow15 while KEEPING occupied20 and exact border10..20.
    one_line(
        &compute::<S>(&paragraph, 20.0, 0.0, 0.0),
        &paragraph,
        20.0,
        0.0,
    );
    one_line(
        &compute::<S>(&paragraph, 20.0, 0.0, -5.0),
        &paragraph,
        20.0,
        -5.0,
    );
    // A genuine before-atomic opportunity permits a different
    //2-line result if dual occupied admission is wrong. No overflow fallback
    // or fabricated indivisibility is used as evidence for success/failure.
    one_line(
        &compute::<S>(&paragraph, 15.0, 0.0, -5.0),
        &paragraph,
        15.0,
        -5.0,
    );
}

#[test]
fn prepared_a_then_real_atomic_negative_end_margin_fits_by_margin_edge_f32() {
    atomic_margin_lane::<f32>();
}
#[test]
fn prepared_a_then_real_atomic_negative_end_margin_fits_by_margin_edge_f64() {
    atomic_margin_lane::<f64>();
}

#[test]
fn direct_native_allocation_keeps_occupied10_when_its_unrelated_flow_is6() {
    // Smallest existing owning fixture route: actual Source InlineBox/issued
    // PreparedAtomic via PreparedParagraph. There is NO Layout node/margin,
    // CSS margin token or inference from advance-occupied. The native caller
    // explicitly supplies two different truthful geometry observations.
    let p = PreparedParagraph::new(
        vec![ParagraphPart::Atomic {
            node: 2,
            size: text::Size::new(10.0, 10.0),
        }],
        ahem_style(10.0, 10.0),
        ParagraphBaseDirection::LeftToRight,
    );
    let prepared = &p.prepared;
    let atomic = prepared.atomic_views().next().unwrap().handle();
    assert_eq!(
        atomic.view().original_box().size(),
        text::Size::new(10.0, 10.0)
    );
    let segment = prepared.source_segments().next().unwrap();
    let line = prepared
        .select(text::LineSelectionRequest {
            inline_boxes: None,
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![text::AtomicAllocation {
                occurrence: atomic,
                allocation: text::AtomicLineAllocation::InFlow {
                    fit: surgeist_text::AtomicFitAllocation::NativeOccupied,
                    flow: text::AtomicFlowAllocation {
                        advance: 6.0,
                        before_baseline: 8.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 10.0,
                    },
                    box_geometry: text::AtomicBoxAllocation {
                        border: text::Rect::new(0.0, -8.0, 10.0, 10.0),
                        content: text::Rect::new(0.0, -8.0, 10.0, 10.0),
                        baseline: 0.0,
                    },
                    alignment: text::LineBoxAlignment::Baseline,
                },
            }],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(6.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.measurements().fit().advance(), 6.0);
    assert_eq!(
        (
            line.measurements().fit().occupied_start(),
            line.measurements().fit().occupied_end()
        ),
        (0.0, 10.0)
    );
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().occupied_start,
            line.feedback().occupied_end
        ),
        (6.0, 0.0, 10.0)
    );
    // Selection/finalization isn't the fit chooser: this doesn't assert that
    // an opaque6px caller band may accept10px. It pins the independent10px
    // native requirement until the FUTURE owner-aware fit observation exists.
}

#[path = "atomic_margin_fit/authority.rs"]
mod authority;
#[path = "atomic_margin_fit/signed.rs"]
mod signed;

#[path = "atomic_margin_fit/composition.rs"]
mod composition;

#[path = "atomic_margin_fit/indent.rs"]
mod indent;
