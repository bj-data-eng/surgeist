//! Real float-height retry, intrinsic sizing and conditional atomic composition.

use super::super::zone_composition::RecessedShape;
use super::*;

fn source_then_atomic(source: &str, size: f32, overflow: text::OverflowWrap) -> PreparedParagraph {
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::Word,
        overflow_wrap: overflow,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        ..ahem_style(size, f64::from(size))
    };
    PreparedParagraph::new(
        vec![
            ParagraphPart::text(
                1,
                Source::identified(source, text::Id::from_u64(72_163), 1),
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

fn tree<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    available: AvailableOf<S>,
    end_margin: f64,
) -> PublicLayoutTreeOf<S> {
    let width = match available {
        AvailableOf::Definite(value) => PreferredSizeOf::px(value),
        _ => PreferredSizeOf::AUTO,
    };
    PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(width, PreferredSizeOf::AUTO),
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
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
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                margin: Edges {
                    right: LengthAutoOf::px(S::from_f64(end_margin)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
}

fn ordinary_batch<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    available: AvailableOf<S>,
    end_margin: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let host = tree(paragraph, available, end_margin);
    let capture = LayoutFlowInput::try_new(
        &host,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
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

fn owned_line<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    index: usize,
) -> text::FinalizedLine {
    batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().line_index() == index)
        .expect("each requested line here contains an actual A origin fragment")
        .fragment()
        .finalized_fragment()
        .finalized_line()
}

fn css_receipt(line: &text::FinalizedLine, advance: f64, raw_end: f64) {
    // Raw occupied observations remain Native observations; this new operation
    // does not overwrite them merely because CSS admission uses margin edges.
    assert_eq!(line.measurements().fit().advance(), advance);
    assert_eq!(
        (
            line.measurements().fit().occupied_start(),
            line.measurements().fit().occupied_end()
        ),
        (0.0, raw_end)
    );
    let fit = line.measurements().margin_edge_fit();
    assert_eq!(fit.authority(), text::MarginEdgeFitAuthority::CssComposed);
    assert_eq!(
        (
            fit.advance(),
            fit.constraint_start(),
            fit.constraint_end(),
            fit.required_inline_size()
        ),
        (advance, 0.0, advance, advance)
    );
}

fn actual_slot(
    line: &text::FinalizedLine,
    paragraph: &PreparedParagraph,
    advance: f64,
    end_margin: f64,
) {
    let slots = line.atomic_slots().collect::<Vec<_>>();
    let [slot] = slots.as_slice() else {
        panic!("one actual native atomic allocation")
    };
    assert_eq!(
        slot.occurrence,
        paragraph.prepared.atomic_views().next().unwrap().handle()
    );
    let text::AtomicLineAllocation::InFlow {
        fit,
        flow,
        box_geometry,
        ..
    } = slot.allocation
    else {
        panic!("the actual tree InlineBlock is in-flow")
    };
    assert_eq!(
        fit,
        text::AtomicFitAllocation::CssMarginBox(
            text::CssMarginBoxFit::try_new(0.0, 10.0, end_margin).unwrap()
        )
    );
    assert_eq!(
        (flow.advance, flow.occupied_start, flow.occupied_end),
        (advance, 0.0, 10.0)
    );
    assert_eq!((flow.before_baseline, flow.after_baseline), (10.0, 0.0));
    assert_eq!(box_geometry.border.size, text::Size::new(10.0, 10.0));
}

fn ahem_source_packet(line: &text::FinalizedLine, size: f64) -> Vec<(text::Range, u32, f64)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(f64::from(run.font_size()), size);
            let instance = run.effective_instance().unwrap();
            let metrics = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!(
                (metrics.ascent(), metrics.descent()),
                (size * 0.8, -size * 0.2)
            );
            run.glyphs()
                .map(|g| (g.range(), g.id(), f64::from(g.advance())))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn changed_float_band<S: LayoutScalar>() {
    let paragraph = paragraph();
    actual_source_premises(&paragraph);
    let available = AvailableOf::definite(S::from_f64(55.0));
    let host = RecessedShape {
        tree: tree(&paragraph, available, -5.0)
            .children(0, [30, 1, 2])
            .children(30, [])
            .style(
                30,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    float_exclusion: FloatExclusion::Shape,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(40.0)),
                        PreferredSizeOf::px(S::from_f64(30.0)),
                    ),
                    ..NodeInputOf::default()
                },
            ),
        queries: Default::default(),
    };
    // Qualify the genuinely height-sensitive provider before observing a run.
    let margin =
        ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(40.0), S::from_f64(30.0)))
            .unwrap();
    for (height, expected) in [(10.0, None), (12.0, Some((0.0, 40.0)))] {
        let query = FloatExclusionQueryOf::try_new(
            margin,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            S::ZERO,
            S::from_f64(height),
        )
        .unwrap();
        let actual = host.float_exclusion_interval(30, query).unwrap().unwrap();
        assert_eq!(
            actual.map(|i| (i.minimum().to_f64(), i.maximum().to_f64())),
            expected
        );
    }
    host.queries.borrow_mut().clear();
    let capture = LayoutFlowInput::try_new(
        &host,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.lines().len(), 1);
    let line = whole_line(&batch);
    assert_eq!(line.prepared_text(), paragraph.prepared);
    let segment = paragraph.prepared.source_segments().next().unwrap();
    assert_eq!(
        (
            line.selected_line().request().start.clone(),
            line.selected_line().request().end.clone()
        ),
        (segment.start(), segment.end())
    );
    assert_eq!(
        line.selected_line().request().context.available_inline,
        Some(15.0)
    );
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (10.0, 12.0)
    );
    assert_eq!(
        ahem_source_packet(&line, 10.0),
        [(text::Range::new(0, 1), 35, 10.0)]
    );
    css_receipt(&line, 15.0, 20.0);
    actual_slot(&line, &paragraph, 5.0, -5.0);
    assert_eq!(
        placed.lines()[0].bounds(),
        text::Rect::new(40.0, 0.0, 15.0, 12.0)
    );
    assert_eq!(placed.lines()[0].baseline(), 10.0);
    assert_eq!(
        placed.inline_boxes()[0].rect(),
        text::Rect::new(50.0, 0.0, 10.0, 10.0)
    );
    let range = placed
        .source_range_fragments(&paragraph.origin(1), text::Range::new(0, 1))
        .unwrap();
    let rects = range
        .units()
        .flat_map(|u| u.pieces().map(|p| p.rect()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(rects, [text::Rect::new(40.0, 2.0, 10.0, 10.0)]);
    let queries = host.queries.borrow();
    // No call-count proxy: actual differently intersecting bands were queried.
    // The provisional candidate starts at zero height; other genuine candidate
    // trials may also inspect the Ahem-only ten-pixel band.
    assert!(queries.iter().any(|(q, hit)| !hit
        && q.band_minimum() == S::ZERO
        && q.band_maximum() <= S::from_f64(10.0)));
    assert!(queries.iter().any(|(q, hit)| *hit
        && q.band_minimum() == S::ZERO
        && q.band_maximum() == S::from_f64(12.0)));
    // Border right60 exceeds root55 by the actual negative end margin. This
    // does not claim border containment: its left50 remains outside float0..40.
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let atomic = entries.iter().find(|e| e.node() == 2).unwrap().output();
        assert_eq!(atomic.location, Point::new(S::from_f64(50.0), S::ZERO));
        assert_eq!(atomic.margin.right, S::from_f64(-5.0));
    }
}

fn intrinsic_paragraph<S: LayoutScalar>() {
    let paragraph = source_then_atomic("A", 5.0, text::OverflowWrap::Normal);
    let before = paragraph
        .prepared
        .opportunity_views()
        .find(|o| o.boundary().view().participant_offset() == 1)
        .unwrap();
    assert_eq!(before.kind(), text::PreparedBreakKind::Allowed);
    assert_eq!(
        paragraph
            .prepared
            .atomic_views()
            .next()
            .unwrap()
            .source_position()
            .get(),
        1
    );
    // Intrinsic paragraph aggregation is over actual legal pieces. Ahem A is5;
    // the actual used atomic margin box is10-4=6. Thus min=max(5,6)=6,
    // max=5+6=11, while the max-content border occupancy still extends to15.
    let minimum = ordinary_batch::<S>(&paragraph, AvailableOf::MIN_CONTENT, -4.0);
    let maximum = ordinary_batch::<S>(&paragraph, AvailableOf::MAX_CONTENT, -4.0);
    for (batch, expected) in [(&minimum, 6.0), (&maximum, 11.0)] {
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let root = entries.iter().find(|e| e.node() == 0).unwrap().output();
            assert_eq!(root.size.width.to_f64(), expected);
            let atomic = entries.iter().find(|e| e.node() == 2).unwrap().output();
            assert_eq!(atomic.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
            assert_eq!(atomic.margin.right, S::from_f64(-4.0));
        }
        assert_eq!(
            batch.placed_text()[0].layout().source().text(),
            &text::Text::from("A")
        );
    }
    assert_eq!(minimum.placed_text()[0].layout().lines().len(), 2);
    assert_eq!(maximum.placed_text()[0].layout().lines().len(), 1);
    let line = whole_line(&maximum);
    assert_eq!(
        ahem_source_packet(&line, 5.0),
        [(text::Range::new(0, 1), 35, 5.0)]
    );
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (10.0, 11.0)
    );
    actual_slot(&line, &paragraph, 6.0, -4.0);
    css_receipt(&line, 11.0, 15.0);
    assert_eq!(
        line.measurements()
            .margin_edge_max_content_contribution()
            .required_inline_size(),
        11.0
    );
    // These are receipts for THIS selected range, not the paragraph's min6.
    assert_eq!(
        line.measurements()
            .margin_edge_min_content_contribution()
            .required_inline_size(),
        11.0
    );
    assert_eq!(
        maximum.placed_text()[0].layout().inline_boxes()[0].rect(),
        text::Rect::new(5.0, 0.0, 10.0, 10.0)
    );
}

fn conditional_then_css_fit<S: LayoutScalar>() {
    let paragraph = source_then_atomic("AA", 10.0, text::OverflowWrap::Anywhere);
    let ordinary = paragraph
        .prepared
        .opportunity_views()
        .find(|o| o.boundary().view().participant_offset() == 2)
        .unwrap();
    assert_eq!(ordinary.kind(), text::PreparedBreakKind::Allowed);
    assert_eq!(ordinary.boundary().view().source_before().get(), 2);
    let batch = ordinary_batch::<S>(&paragraph, AvailableOf::definite(S::from_f64(16.0)), -4.0);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.lines().len(), 2);
    let first = owned_line(&batch, 0);
    let second = owned_line(&batch, 1);
    let first_selected = first.selected_line();
    let text::PreparedTermination::Conditional(cut) = &first_selected.request().termination else {
        panic!(
            "AA20 cannot fit16 at the ordinary before-atomic cut; first A requires actual anywhere authority"
        )
    };
    assert_eq!(cut.prepared_text(), paragraph.prepared);
    assert_eq!(cut.purpose(), text::ConditionalPurpose::Overflow);
    assert_eq!(cut.boundary().view().source_before().get(), 1);
    assert_eq!(cut.boundary().view().participant_offset(), 1);
    assert_eq!(first_selected.request().context.end, text::LineEnd::Soft);
    assert_eq!(
        second.selected_line().request().start,
        first_selected.request().end
    );
    assert_eq!(
        second.selected_line().request().context.end,
        text::LineEnd::Paragraph
    );
    assert_eq!(
        ahem_source_packet(&first, 10.0),
        [(text::Range::new(0, 1), 35, 10.0)]
    );
    assert_eq!(
        ahem_source_packet(&second, 10.0),
        [(text::Range::new(1, 2), 35, 10.0)]
    );
    assert_eq!(first.feedback().advance, 10.0);
    assert_eq!(first.atomic_slots().count(), 0);
    css_receipt(&second, 16.0, 20.0);
    actual_slot(&second, &paragraph, 6.0, -4.0);
    assert_eq!(
        placed.lines()[0].bounds(),
        text::Rect::new(0.0, 0.0, 16.0, 10.0)
    );
    assert_eq!(
        placed.lines()[1].bounds(),
        text::Rect::new(0.0, 10.0, 16.0, 12.0)
    );
    assert_eq!(
        (placed.lines()[0].baseline(), placed.lines()[1].baseline()),
        (8.0, 20.0)
    );
    assert_eq!(
        placed.inline_boxes()[0].rect(),
        text::Rect::new(10.0, 10.0, 10.0, 10.0)
    );
}

#[test]
fn actual_float_height_retry_preserves_signed_css_fit_f32() {
    changed_float_band::<f32>();
}
#[test]
fn actual_float_height_retry_preserves_signed_css_fit_f64() {
    changed_float_band::<f64>();
}
#[test]
fn paragraph_intrinsic_chooser_uses_signed_atomic_margin_pieces_f32() {
    intrinsic_paragraph::<f32>();
}
#[test]
fn paragraph_intrinsic_chooser_uses_signed_atomic_margin_pieces_f64() {
    intrinsic_paragraph::<f64>();
}
#[test]
fn real_anywhere_cut_then_margin_box_paragraph_end_fits_f32() {
    conditional_then_css_fit::<f32>();
}
#[test]
fn real_anywhere_cut_then_margin_box_paragraph_end_fits_f64() {
    conditional_then_css_fit::<f64>();
}
