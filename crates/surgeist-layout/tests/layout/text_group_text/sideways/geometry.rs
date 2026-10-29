//! Physical frames are literal WM4/CW projections of real Ahem line receipts.
use super::*;

pub(super) fn px<S: l::LayoutScalar>(value: f64) -> l::PreferredSizeOf<S> {
    l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(value)).unwrap())
}
pub(super) fn fixture<S: l::LayoutScalar>(
    direction: l::Direction,
    width: Option<f64>,
    height: Option<f64>,
) -> Host<S> {
    let mut host = Host::new(direction, l::TextGroupAlign::None);
    host.box_change(0, |root| {
        root.writing_mode = l::WritingMode::SidewaysRl;
        root.text_align = l::TextAlign::Start;
        root.inline_baseline_context = l::InlineBaselineContext::Physical;
        root.size = l::Size::new(
            width.map_or(l::PreferredSizeOf::AUTO, px),
            height.map_or(l::PreferredSizeOf::AUTO, px),
        );
    });
    host
}
pub(super) fn paragraph<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parts: &[Part<'_>],
    text_style: t::Style,
    body: t::ParagraphBaseDirection,
) -> Paragraph {
    Paragraph::with_preparation(
        host,
        container,
        parts,
        text_style,
        body,
        t::PreparationOptions::default().with_orientation(t::LineOrientation::SidewaysClockwise),
    )
}
pub(super) fn available<S: l::LayoutScalar>(width: f64, height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(S::from_f64(width)),
        l::AvailableOf::definite(S::from_f64(height)),
    )
}
pub(super) fn run<S: l::LayoutScalar>(
    host: &Host<S>,
    paragraphs: &[&Paragraph],
    available: l::Size<l::AvailableOf<S>>,
    changed: &[u32],
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(
        host,
        0,
        paragraphs.iter().map(|p| p.binding.clone()).collect(),
        l::WrappingInput::default(),
    )
    .unwrap();
    let request = l::LayoutRootRequestOf::viewport(available).unwrap();
    let batch = l::compute_layout_with_flow_invalidated(&flow, 0, request, changed).unwrap();
    // These core fixtures use integer physical boxes. Final publication must
    // retain the actual unrounded box geometry and native source custody.
    for final_row in batch.final_entries() {
        let raw = batch
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == final_row.node())
            .unwrap()
            .output();
        let final_output = final_row.output();
        assert_eq!(final_output.location, raw.location);
        assert_eq!(final_output.size, raw.size);
        assert_eq!(final_output.source_index, raw.source_index);
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .filter(|row| row.node() == final_row.node())
                .count(),
            1
        );
    }
    batch
}
pub(super) fn break_all() -> t::Style {
    t::Style {
        word_break: t::WordBreak::BreakAll,
        ..style()
    }
}

fn two_columns<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    p: &Paragraph,
    right: f64,
    top: f64,
    inline_start: f64,
    capacity: f64,
    request_capacity: Option<f64>,
) {
    let layout = placed(batch, p);
    assert_eq!(layout.orientation(), t::LineOrientation::SidewaysClockwise);
    assert_eq!(layout.source().text().code_units(), [65, 66]);
    assert_eq!(
        layout.analyzed_source().identity(),
        p.prepared.analyzed_source().identity()
    );
    let lines = receipts(batch, p);
    assert_eq!(lines.len(), 2);
    for (i, line) in lines.iter().enumerate() {
        assert_eq!(line.prepared_text(), p.prepared);
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.feedback().line_height, 10.0);
        assert_eq!(
            line.selected_line().request().context.available_inline,
            request_capacity
        );
        assert_eq!(
            line.selected_line()
                .request()
                .end
                .view()
                .source_after()
                .get(),
            i + 1
        );
        assert_eq!(
            line.selected_line().request().context.end,
            if i == 0 {
                t::LineEnd::Soft
            } else {
                t::LineEnd::Paragraph
            }
        );
    }
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| (line.baseline(), line.bounds()))
            .collect::<Vec<_>>(),
        [
            (right - 8.0, t::Rect::new(right - 10.0, top, 10.0, capacity)),
            (
                right - 18.0,
                t::Rect::new(right - 20.0, top, 10.0, capacity)
            ),
        ]
    );
    assert_eq!(
        layout
            .run_views()
            .map(|run| run.origin())
            .collect::<Vec<_>>(),
        [
            t::Point::new(right - 8.0, inline_start),
            t::Point::new(right - 18.0, inline_start)
        ]
    );
    assert_eq!(
        layout
            .run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|g| (g.id(), g.advance(), run.range()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            (35, 10.0, t::Range::new(0, 1)),
            (36, 10.0, t::Range::new(1, 2))
        ]
    );
    assert_eq!(layout.metrics().size(), t::Size::new(20.0, 10.0));
}

fn auto_and_intrinsic<S: l::LayoutScalar>() {
    // A finite outer40-wide Srl frame anchors an AUTO20-wide root at GX20.
    // A nonfinite block request instead anchors it at GX0. Both are actual
    // root requests; no provisional-width frame is supplied by the test.
    for (block_space, gx, right) in [
        (l::AvailableOf::definite(S::from_f64(40.0)), 20.0, 40.0),
        (l::AvailableOf::MAX_CONTENT, 0.0, 20.0),
    ] {
        let mut host = fixture::<S>(l::Direction::Ltr, None, Some(15.0));
        let p = paragraph(
            &mut host,
            0,
            &[Part::Text("AB")],
            break_all(),
            t::ParagraphBaseDirection::LeftToRight,
        );
        let batch = run(
            &host,
            &[&p],
            l::Size::new(block_space, l::AvailableOf::definite(S::from_f64(15.0))),
            &[],
        );
        let root = node_output(&batch, 0);
        assert_eq!(root.location, l::Point::new(S::from_f64(gx), S::ZERO));
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(20.0), S::from_f64(15.0))
        );
        two_columns(&batch, &p, right, 0.0, 0.0, 15.0, Some(15.0));
    }
    // These are real Layout MIN/MAX_CONTENT requests along physical height
    // (Srl inline), not a claim that one whole-line measurement is global min.
    for (inline_space, width, height) in [
        (l::AvailableOf::MIN_CONTENT, 20.0, 10.0),
        (l::AvailableOf::MAX_CONTENT, 10.0, 20.0),
    ] {
        let mut host = fixture::<S>(l::Direction::Ltr, None, None);
        let p = paragraph(
            &mut host,
            0,
            &[Part::Text("AB")],
            break_all(),
            t::ParagraphBaseDirection::LeftToRight,
        );
        let batch = run(
            &host,
            &[&p],
            l::Size::new(l::AvailableOf::MAX_CONTENT, inline_space),
            &[],
        );
        let root = node_output(&batch, 0);
        assert_eq!(root.location, l::Point::ZERO);
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(width), S::from_f64(height))
        );
        if width == 20.0 {
            // The initial MIN_CONTENT measurement has no finite allocation.
            // Final Block placement uses the actual settled10-unit inline
            // frame; this packet therefore retains an exact finite capacity.
            two_columns(&batch, &p, 20.0, 0.0, 0.0, 10.0, Some(10.0));
            for line in receipts(&batch, &p) {
                assert_eq!(
                    line.measurements().min_content_contribution().advance(),
                    10.0
                );
                assert_eq!(
                    line.measurements().max_content_contribution().advance(),
                    10.0
                );
            }
        } else {
            let layout = placed(&batch, &p);
            assert_eq!(
                receipts(&batch, &p)
                    .iter()
                    .map(|r| r.feedback().advance)
                    .collect::<Vec<_>>(),
                [20.0]
            );
            let rows = receipts(&batch, &p);
            assert_eq!(
                rows[0].selected_line().request().context.available_inline,
                Some(20.0)
            );
            assert_eq!(
                rows[0].measurements().max_content_contribution().advance(),
                20.0
            );
            assert_eq!(
                rows[0]
                    .selected_line()
                    .request()
                    .end
                    .view()
                    .source_after()
                    .get(),
                2
            );
            assert_eq!(
                rows[0].selected_line().request().context.end,
                t::LineEnd::Paragraph
            );
            assert_eq!(layout.lines()[0].baseline(), 2.0);
            assert_eq!(
                layout.lines()[0].bounds(),
                t::Rect::new(0.0, 0.0, 10.0, 20.0)
            );
            assert_eq!(
                layout.run_views().next().unwrap().origin(),
                t::Point::new(2.0, 0.0)
            );
        }
    }
}
#[test]
fn auto_root_origin_and_actual_intrinsic_requests_f32() {
    auto_and_intrinsic::<f32>();
}
#[test]
fn auto_root_origin_and_actual_intrinsic_requests_f64() {
    auto_and_intrinsic::<f64>();
}

fn insets_and_ancestor<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for body in [
            t::ParagraphBaseDirection::LeftToRight,
            t::ParagraphBaseDirection::RightToLeft,
        ] {
            let mut host = fixture::<S>(direction, Some(50.0), Some(25.0));
            host.box_change(0, |root| {
                root.box_sizing = l::BoxSizing::BorderBox;
                root.padding.left =
                    l::LengthOf::value(l::LengthPercentageOf::px(S::from_f64(3.0)).unwrap());
                root.padding.right =
                    l::LengthOf::value(l::LengthPercentageOf::px(S::from_f64(7.0)).unwrap());
                root.padding.top =
                    l::LengthOf::value(l::LengthPercentageOf::px(S::from_f64(4.0)).unwrap());
                root.padding.bottom =
                    l::LengthOf::value(l::LengthPercentageOf::px(S::from_f64(6.0)).unwrap());
            });
            let p = paragraph(&mut host, 0, &[Part::Text("AB")], break_all(), body);
            let expected_level = if body == t::ParagraphBaseDirection::LeftToRight {
                0
            } else {
                2
            };
            assert!(
                p.prepared
                    .unit_views()
                    .all(|u| u.level().get() == expected_level)
            );
            let batch = run(&host, &[&p], available(50.0, 25.0), &[]);
            assert_eq!(
                node_output(&batch, 0).size,
                l::Size::new(S::from_f64(50.0), S::from_f64(25.0))
            );
            // Content is x3..43/y4..19. RTL Start shifts the10-unit glyph
            // down5; it neither changes Latin's actual UBA order nor the band.
            two_columns(
                &batch,
                &p,
                43.0,
                4.0,
                if direction == l::Direction::Ltr {
                    4.0
                } else {
                    9.0
                },
                15.0,
                Some(15.0),
            );
            for row in receipts(&batch, &p) {
                assert_eq!(row.selected_line().request().context.base_direction, body);
                assert_eq!(
                    row.selected_line().request().context.line_box_direction,
                    if direction == l::Direction::Ltr {
                        t::LineBoxDirection::LeftToRight
                    } else {
                        t::LineBoxDirection::RightToLeft
                    }
                );
            }
        }
    }
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(0, |root| {
        root.padding.left =
            l::LengthOf::value(l::LengthPercentageOf::px(S::from_f64(7.0)).unwrap());
        root.padding.top = l::LengthOf::value(l::LengthPercentageOf::px(S::from_f64(9.0)).unwrap());
    });
    let child = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(child, |input| {
        input.writing_mode = l::WritingMode::SidewaysRl;
        input.size = l::Size::new(px(40.0), px(15.0));
    });
    let p = paragraph(
        &mut host,
        child,
        &[Part::Text("AB")],
        break_all(),
        t::ParagraphBaseDirection::LeftToRight,
    );
    let batch = run(&host, &[&p], available(100.0, 100.0), &[]);
    assert_eq!(
        node_output(&batch, child).location,
        l::Point::new(S::from_f64(7.0), S::from_f64(9.0))
    );
    two_columns(&batch, &p, 47.0, 9.0, 9.0, 15.0, Some(15.0));
}
#[test]
fn actual_insets_ancestor_and_independent_linebox_direction_f32() {
    insets_and_ancestor::<f32>();
}
#[test]
fn actual_insets_ancestor_and_independent_linebox_direction_f64() {
    insets_and_ancestor::<f64>();
}

fn cache_and_change<S: l::LayoutScalar>() {
    let mut host = fixture::<S>(l::Direction::Ltr, Some(40.0), Some(15.0));
    // Complete BlockEnd belongs to this real paragraph container only. The
    // eligible cache leaf is a separate normal block under the outer root,
    // never an omitted out-of-flow member of the captured native inline run.
    let text_box = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(text_box, |input| {
        input.writing_mode = l::WritingMode::SidewaysRl;
        input.size = l::Size::new(l::PreferredSizeOf::AUTO, px(15.0));
    });
    let p = paragraph(
        &mut host,
        text_box,
        &[Part::Text("AB")],
        break_all(),
        t::ParagraphBaseDirection::LeftToRight,
    );
    let leaf = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(leaf, |input| {
        input.writing_mode = l::WritingMode::SidewaysRl;
        input.block_flow_context = l::BlockFlowContext::Independent;
        input.size = l::Size::new(px(5.0), px(15.0));
    });
    assert_eq!(host.children[0], [text_box, leaf]);
    assert_eq!(host.children[text_box as usize], p.binding.nodes);
    let cold = run(&host, &[&p], available(40.0, 15.0), &[]);
    // Two10-unit columns use20 block units. At block-start/right of40,
    // their actual container is x20..40; its next5-unit sibling is x15..20.
    assert_eq!(
        node_output(&cold, text_box).location,
        l::Point::new(S::from_f64(20.0), S::ZERO)
    );
    assert_eq!(
        node_output(&cold, text_box).size,
        l::Size::new(S::from_f64(20.0), S::from_f64(15.0))
    );
    assert_eq!(
        node_output(&cold, leaf).size,
        l::Size::new(S::from_f64(5.0), S::from_f64(15.0))
    );
    two_columns(&cold, &p, 40.0, 0.0, 0.0, 15.0, Some(15.0));
    assert_eq!(
        node_output(&cold, leaf).location,
        l::Point::new(S::from_f64(15.0), S::ZERO)
    );
    cold.apply_to(&mut host).unwrap();
    let before = host.hits[leaf as usize].get();
    let warm = run(&host, &[&p], available(40.0, 15.0), &[]);
    two_columns(&warm, &p, 40.0, 0.0, 0.0, 15.0, Some(15.0));
    assert!(
        host.hits[leaf as usize].get() > before,
        "actual eligible block leaf cache must be reused; Text root replay alone is not warm-cache proof"
    );
    warm.apply_to(&mut host).unwrap();
    host.box_change(0, |input| input.size.width = px(50.0));
    let changed = run(&host, &[&p], available(50.0, 15.0), &[0]);
    assert_eq!(
        node_output(&changed, text_box).location,
        l::Point::new(S::from_f64(30.0), S::ZERO)
    );
    assert_eq!(
        node_output(&changed, text_box).size,
        l::Size::new(S::from_f64(20.0), S::from_f64(15.0))
    );
    two_columns(&changed, &p, 50.0, 0.0, 0.0, 15.0, Some(15.0));
    assert_eq!(
        node_output(&changed, leaf).location,
        l::Point::new(S::from_f64(25.0), S::ZERO)
    );
    changed.apply_to(&mut host).unwrap();
    let current = run(&host, &[&p], available(50.0, 15.0), &[]);
    two_columns(&current, &p, 50.0, 0.0, 0.0, 15.0, Some(15.0));
    // Both previously published packets retain their original global frame.
    two_columns(&cold, &p, 40.0, 0.0, 0.0, 15.0, Some(15.0));
    two_columns(&warm, &p, 40.0, 0.0, 0.0, 15.0, Some(15.0));
    let mut fresh = fixture::<S>(l::Direction::Ltr, Some(50.0), Some(15.0));
    let fresh_text_box = fresh.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    fresh.box_change(fresh_text_box, |input| {
        input.writing_mode = l::WritingMode::SidewaysRl;
        input.size = l::Size::new(l::PreferredSizeOf::AUTO, px(15.0));
    });
    let fresh_p = paragraph(
        &mut fresh,
        fresh_text_box,
        &[Part::Text("AB")],
        break_all(),
        t::ParagraphBaseDirection::LeftToRight,
    );
    let fresh_leaf = fresh.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    fresh.box_change(fresh_leaf, |input| {
        input.writing_mode = l::WritingMode::SidewaysRl;
        input.block_flow_context = l::BlockFlowContext::Independent;
        input.size = l::Size::new(px(5.0), px(15.0));
    });
    let fresh_batch = run(&fresh, &[&fresh_p], available(50.0, 15.0), &[]);
    two_columns(&fresh_batch, &fresh_p, 50.0, 0.0, 0.0, 15.0, Some(15.0));
    assert_eq!(
        node_output(&current, leaf),
        node_output(&fresh_batch, fresh_leaf)
    );
}
#[test]
fn real_eligible_leaf_cache_replay_width_invalidation_and_old_packets_f32() {
    cache_and_change::<f32>();
}
#[test]
fn real_eligible_leaf_cache_replay_width_invalidation_and_old_packets_f64() {
    cache_and_change::<f64>();
}

fn quality_continuation<S: l::LayoutScalar>() {
    let mut host = fixture::<S>(l::Direction::Ltr, Some(40.0), Some(60.0));
    let p = paragraph(
        &mut host,
        0,
        &[Part::Text("AAA BB C")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
    );
    let mut retained = Vec::new();
    for (mode, cuts, advances) in [
        (l::TextWrapStyle::Auto, [7, 8], [60.0, 10.0]),
        (l::TextWrapStyle::Stable, [7, 8], [60.0, 10.0]),
        (l::TextWrapStyle::Balance, [4, 8], [30.0, 40.0]),
        (l::TextWrapStyle::AvoidShortLastLine, [4, 8], [30.0, 40.0]),
        (l::TextWrapStyle::Pretty, [4, 8], [30.0, 40.0]),
    ] {
        host.box_change(0, |root| root.text_wrap_style = mode);
        let batch = run(&host, &[&p], available(40.0, 60.0), &[0]);
        let layout = placed(&batch, &p);
        assert_eq!(
            layout.source().text().code_units(),
            [65, 65, 65, 32, 66, 66, 32, 67]
        );
        let rows = receipts(&batch, &p);
        assert_eq!(rows.len(), 2);
        for (i, row) in rows.iter().enumerate() {
            let selected = row.selected_line();
            let request = selected.request();
            assert_eq!(row.prepared_text(), p.prepared);
            assert_eq!(request.end.view().source_after().get(), cuts[i]);
            assert_eq!(request.context.available_inline, Some(60.0));
            assert_eq!(
                request.context.start,
                if i == 0 {
                    t::LineStart::Paragraph
                } else {
                    t::LineStart::Continuation
                }
            );
            assert_eq!(
                request.context.end,
                if i == 0 {
                    t::LineEnd::Soft
                } else {
                    t::LineEnd::Paragraph
                }
            );
            assert!(request.alternative.is_none());
            assert_eq!(row.measurements().alignment().advance(), advances[i]);
            assert_eq!(
                layout.lines()[i].baseline(),
                if i == 0 { 32.0 } else { 22.0 }
            );
            assert_eq!(
                layout.lines()[i].bounds(),
                t::Rect::new(if i == 0 { 30.0 } else { 20.0 }, 0.0, 10.0, 60.0)
            );
        }
        assert_eq!(
            rows[1].selected_line().request().start,
            rows[0].selected_line().request().end
        );
        batch.apply_to(&mut host).unwrap();
        retained.push((batch, cuts, advances));
    }
    // Each earlier immutable publication retains its actual issued selections;
    // equality of freshly minted FinalizedLine handles is not a semantic oracle.
    for (batch, cuts, advances) in retained {
        let rows = receipts(&batch, &p);
        assert_eq!(
            rows.iter()
                .map(|r| r.selected_line().request().end.view().source_after().get())
                .collect::<Vec<_>>(),
            cuts
        );
        assert_eq!(
            rows.iter()
                .map(|r| r.measurements().alignment().advance())
                .collect::<Vec<_>>(),
            advances
        );
    }
}
#[test]
fn real_quality_modes_keep_cw_frames_and_selected_continuations_f32() {
    quality_continuation::<f32>();
}
#[test]
fn real_quality_modes_keep_cw_frames_and_selected_continuations_f64() {
    quality_continuation::<f64>();
}
