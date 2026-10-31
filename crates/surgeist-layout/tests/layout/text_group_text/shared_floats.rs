//! Ordinary shared-BFC float custody through a genuine Text consumer.
use super::*;

fn px<S: l::LayoutScalar>(value: f64) -> l::PreferredSizeOf<S> {
    l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn unique_output<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> l::NodeOutputOf<S> {
    let mut found = entries.iter().filter(|row| row.node() == node);
    let output = found.next().unwrap().output();
    assert!(found.next().is_none(), "one packet per actual node");
    output
}

fn assert_shared_text_packet<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
    ordinary: u32,
    float: u32,
    later: u32,
    offset: (f64, f64),
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = unique_output(entries, 0);
        let shared = unique_output(entries, ordinary);
        let actual_float = unique_output(entries, float);
        let text_block = unique_output(entries, later);
        // CSS2.1 10.6.3 excludes the float from this ordinary block's height;
        // 10.6.7 includes its nominal bottom30 in the actual root BFC.
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(100.0), S::from_f64(30.0))
        );
        assert_eq!(shared.size, l::Size::new(S::from_f64(100.0), S::ZERO));
        assert_eq!(
            shared.location,
            l::Point::new(S::from_f64(offset.0), S::from_f64(offset.1))
        );
        assert_eq!(actual_float.location, l::Point::ZERO);
        assert_eq!(
            actual_float.size,
            l::Size::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        // Its visible global position moves with the relative ancestor, while
        // later normal-flow lines use the original exclusion, not that shift.
        assert_eq!(
            shared.location.x + actual_float.location.x,
            S::from_f64(offset.0)
        );
        assert_eq!(
            shared.location.y + actual_float.location.y,
            S::from_f64(offset.1)
        );
        assert_eq!(text_block.location, l::Point::ZERO);
        assert_eq!(
            text_block.size,
            l::Size::new(S::from_f64(100.0), S::from_f64(10.0))
        );
        for (node, index) in [(0, 0), (ordinary, 0), (float, 0), (later, 1)] {
            assert_eq!(
                unique_output(entries, node).source_index,
                l::SourceIndex::new(index)
            );
        }
    }

    let layout = placed(batch, paragraph);
    assert_eq!(
        layout.source().text().code_units(),
        "AAA BB C".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.nonformatted_lines().count(), 0);
    assert_eq!(layout.lines()[0].range(), t::Range::new(0, 8));
    assert_eq!(
        layout.lines()[0].bounds(),
        t::Rect::new(20.0, 0.0, 80.0, 10.0)
    );
    assert_eq!(layout.lines()[0].baseline(), 8.0);
    assert_eq!(glyph_extents(layout), [(20.0, 100.0)]);
    for run in layout.run_views() {
        assert_eq!(run.origin().y, 8.0);
    }

    let lines = receipts(batch, paragraph);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    let selected = line.selected_line();
    let request = selected.request();
    assert_eq!(line.prepared_text(), paragraph.prepared);
    assert_eq!(request.start.view().source_after().get(), 0);
    assert_eq!(request.end.view().source_after().get(), 8);
    assert_eq!(request.context.end, t::LineEnd::Paragraph);
    assert_eq!(request.context.available_inline, Some(80.0));
    assert_eq!(request.context.indent, 0.0);
    assert_eq!(
        request.context.alignment_insets,
        t::LineAlignmentInsets::ZERO
    );
    assert!(request.alternative.is_none());
    assert!(request.controls.is_empty());
    assert!(request.atomics.is_empty());
    let t::PreparedTermination::Ordinary(offer) = &request.termination else {
        panic!("the unchanged genuine paragraph ending terminates this fitting source");
    };
    assert_eq!(offer.view().kind(), t::PreparedBreakKind::ParagraphEnd);
    assert_eq!(offer.view().boundary(), request.end);
    let natural = selected
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.measurements().alignment().advance(), 80.0);
    assert_eq!(line.measurements().fit().advance(), 80.0);
    assert_eq!(line.feedback().line_height, 10.0);

    assert_eq!(
        layout
            .cursor(t::Cursor::new(0, t::Affinity::After))
            .unwrap()
            .rect()
            .origin,
        t::Point::new(20.0, 0.0)
    );
    assert_eq!(
        layout
            .cursor(t::Cursor::new(8, t::Affinity::Before))
            .unwrap()
            .rect()
            .origin
            .x,
        100.0
    );
    let selected_word = layout
        .selection(t::Selection::new(
            t::Cursor::new(0, t::Affinity::After),
            t::Cursor::new(3, t::Affinity::Before),
        ))
        .unwrap();
    assert_eq!(selected_word.rects().len(), 1);
    assert_eq!(
        selected_word.rects()[0].rect(),
        t::Rect::new(20.0, 0.0, 30.0, 10.0)
    );
}

fn shared_relative_float_text<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::None);
    let ordinary = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let float = host.push(
        Some(ordinary),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            float: l::Float::Left,
            size: l::Size::new(px(20.0), px(30.0)),
            ..l::NodeInputOf::default()
        }),
    );
    let later = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let paragraph = Paragraph::new(
        &mut host,
        later,
        &[Part::Text("AAA BB C")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let old = compute(&host, &[&paragraph], &[]);
    assert_shared_text_packet(&old, &paragraph, ordinary, float, later, (0.0, 0.0));
    old.apply_to(&mut host).unwrap();

    host.box_change(ordinary, |input| {
        input.position = l::Position::Relative;
        input.inset.left =
            l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(7.0)).unwrap());
        input.inset.top =
            l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(11.0)).unwrap());
    });
    let shifted = compute(&host, &[&paragraph], &[ordinary]);
    assert_shared_text_packet(&shifted, &paragraph, ordinary, float, later, (7.0, 11.0));
    assert!(shifted.invalidated_nodes().contains(&0));
    shifted.apply_to(&mut host).unwrap();
    let replay = compute(&host, &[&paragraph], &[]);
    assert_shared_text_packet(&replay, &paragraph, ordinary, float, later, (7.0, 11.0));
    let mut shifted_rows = shifted
        .final_entries()
        .iter()
        .map(|row| (row.node(), row.output()))
        .collect::<Vec<_>>();
    let mut replay_rows = replay
        .final_entries()
        .iter()
        .map(|row| (row.node(), row.output()))
        .collect::<Vec<_>>();
    shifted_rows.sort_by_key(|row| row.0);
    replay_rows.sort_by_key(|row| row.0);
    assert_eq!(shifted_rows, replay_rows);
    // Existing packets retain their original source, cuts and geometry after
    // actual publication and relative movement of the later captured tree.
    assert_shared_text_packet(&old, &paragraph, ordinary, float, later, (0.0, 0.0));
    assert_shared_text_packet(&shifted, &paragraph, ordinary, float, later, (7.0, 11.0));
}

#[test]
fn shared_descendant_float_shortens_real_sibling_text_without_using_relative_visual_offsets() {
    shared_relative_float_text::<f32>();
    shared_relative_float_text::<f64>();
}
