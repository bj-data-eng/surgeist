use super::*;

fn x_ranges<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
) -> Vec<(f64, f64)> {
    glyph_extents(placed(batch, paragraph))
}
fn descendant_and_nested<S: l::LayoutScalar>() {
    for child_group in [l::TextGroupAlign::None, l::TextGroupAlign::End] {
        let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
        let outside = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
        let inside = host.child_block(0, l::Direction::Ltr, child_group);
        let wide = Paragraph::new(
            &mut host,
            outside,
            &[Part::Text("DDDDDDDD")],
            style(),
            t::ParagraphBaseDirection::LeftToRight,
            false,
        );
        let pair = Paragraph::pair(&mut host, inside);
        let expected = if child_group == l::TextGroupAlign::None {
            10.0
        } else {
            30.0
        };
        for reverse in [false, true] {
            host.children[0] = if reverse {
                vec![inside, outside]
            } else {
                vec![outside, inside]
            };
            let batch = compute(&host, &[&wide, &pair], &[]);
            assert_eq!(x_ranges(&batch, &wide), [(10.0, 90.0)]);
            assert_eq!(
                x_ranges(&batch, &pair),
                [(expected, expected + 60.0), (expected, expected + 40.0)]
            );
            for line in receipts(&batch, &pair) {
                let inset = line.selected_line().request().context.alignment_insets;
                assert_eq!(
                    (inset.line_left(), inset.line_right()),
                    if child_group == l::TextGroupAlign::None {
                        (10.0, 10.0)
                    } else {
                        (30.0, 10.0)
                    }
                );
            }
        }
    }
    // The child's subset contains the parent's actual80-unit limiter. It has
    // no residual padding left after the ancestor's10/10 operation.
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let child = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::End);
    let wide_container = host.child_block(child, l::Direction::Ltr, l::TextGroupAlign::None);
    let small_container = host.child_block(child, l::Direction::Ltr, l::TextGroupAlign::None);
    let wide = Paragraph::new(
        &mut host,
        wide_container,
        &[Part::Text("DDDDDDDD")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let pair = Paragraph::pair(&mut host, small_container);
    let batch = compute(&host, &[&wide, &pair], &[]);
    assert_eq!(x_ranges(&batch, &wide), [(10.0, 90.0)]);
    assert_eq!(x_ranges(&batch, &pair), [(10.0, 70.0), (10.0, 50.0)]);
}
#[test]
fn ancestor_group_includes_real_descendant_none_and_nested_end_spends_only_residual_slack() {
    descendant_and_nested::<f32>();
    descendant_and_nested::<f64>();
}

fn independent<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let normal = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let independent = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(independent, |node| {
        node.block_flow_context = l::BlockFlowContext::Independent
    });
    let pair = Paragraph::pair(&mut host, normal);
    let excluded = Paragraph::new(
        &mut host,
        independent,
        &[Part::Text("CCCCCCCCC")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&pair, &excluded], &[]);
    assert_eq!(x_ranges(&batch, &pair), [(20.0, 80.0), (20.0, 60.0)]);
    assert_eq!(x_ranges(&batch, &excluded), [(0.0, 90.0)]);
    assert_eq!(
        receipts(&batch, &excluded)[0]
            .selected_line()
            .request()
            .context
            .alignment_insets,
        t::LineAlignmentInsets::ZERO
    );
}
#[test]
fn genuine_independent_block_interior_does_not_limit_or_receive_its_ancestor_text_group() {
    independent::<f32>();
    independent::<f64>();
}

fn member_directions<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Start);
    let left = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let right = host.child_block(0, l::Direction::Rtl, l::TextGroupAlign::None);
    let a = Paragraph::new(
        &mut host,
        left,
        &[Part::Text("AAAAAA")],
        style(),
        t::ParagraphBaseDirection::RightToLeft,
        false,
    );
    let b = Paragraph::new(
        &mut host,
        right,
        &[Part::Text("BBBB")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&a, &b], &[]);
    assert_eq!(x_ranges(&batch, &a), [(0.0, 60.0)]);
    assert_eq!(x_ranges(&batch, &b), [(60.0, 100.0)]);
    let a = receipts(&batch, &a);
    let b = receipts(&batch, &b);
    assert_eq!(
        a[0].selected_line().request().context.line_box_direction,
        t::LineBoxDirection::LeftToRight
    );
    assert_eq!(
        b[0].selected_line().request().context.line_box_direction,
        t::LineBoxDirection::RightToLeft
    );
    assert_eq!(
        a[0].selected_line().request().context.base_direction,
        t::ParagraphBaseDirection::RightToLeft
    );
    assert_eq!(
        b[0].selected_line().request().context.base_direction,
        t::ParagraphBaseDirection::LeftToRight
    );
    assert_eq!(
        (
            a[0].selected_line()
                .request()
                .context
                .alignment_insets
                .line_left(),
            a[0].selected_line()
                .request()
                .context
                .alignment_insets
                .line_right()
        ),
        (0.0, 40.0)
    );
    assert_eq!(
        (
            b[0].selected_line()
                .request()
                .context
                .alignment_insets
                .line_left(),
            b[0].selected_line()
                .request()
                .context
                .alignment_insets
                .line_right()
        ),
        (40.0, 0.0)
    );
}
fn plaintext_fallback<S: l::LayoutScalar>(rtl_strong: bool) {
    let containing = if rtl_strong {
        l::Direction::Ltr
    } else {
        l::Direction::Rtl
    };
    let mut host = Host::<S>::new(containing, l::TextGroupAlign::Start);
    host.box_change(0, |root| {
        root.line_box_direction_policy = l::LineBoxDirectionPolicy::Plaintext
    });
    // RLM is genuine strong semantic text with zero advance; no dummy glyph or
    // borrowed numeric level supplies direction. Empty Force and12 have no
    // eligible strong unit and must retain the actual accepted prior fallback.
    let first = if rtl_strong {
        "\u{200f}AA A A"
    } else {
        "AA A A"
    };
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &[
            Part::Text(first),
            Part::Forced,
            Part::Text(""),
            Part::Forced,
            Part::Text("12"),
        ],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&paragraph], &[]);
    let layout = placed(&batch, &paragraph);
    assert_eq!(layout.lines().len(), 3);
    let extents = glyph_extents(layout);
    assert_eq!(
        extents[0],
        if rtl_strong {
            (40.0, 100.0)
        } else {
            (0.0, 60.0)
        }
    );
    assert_eq!(
        extents[2],
        if rtl_strong {
            (80.0, 100.0)
        } else {
            (0.0, 20.0)
        }
    );
    for line in receipts(&batch, &paragraph) {
        let selected = line.selected_line();
        let context = &selected.request().context;
        assert_eq!(
            context.line_box_direction,
            if rtl_strong {
                t::LineBoxDirection::RightToLeft
            } else {
                t::LineBoxDirection::LeftToRight
            }
        );
        assert_eq!(
            (
                context.alignment_insets.line_left(),
                context.alignment_insets.line_right()
            ),
            if rtl_strong { (40.0, 0.0) } else { (0.0, 40.0) }
        );
    }
}
#[test]
fn logical_group_sides_use_each_real_member_direction_and_plaintext_accepted_fallback() {
    member_directions::<f32>();
    member_directions::<f64>();
    for rtl in [false, true] {
        plaintext_fallback::<f32>(rtl);
        plaintext_fallback::<f64>(rtl);
    }
}

fn changed_limiter<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let member = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let limiter = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let cache_leaf = host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(cache_leaf, |node| {
        node.block_flow_context = l::BlockFlowContext::Independent;
        node.size.height =
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(5.0)).unwrap());
    });
    let pair = Paragraph::pair(&mut host, member);
    let old_limiter = Paragraph::new(
        &mut host,
        limiter,
        &[Part::Text("DDDDDDDD")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let old = compute(&host, &[&pair, &old_limiter], &[]);
    assert_eq!(x_ranges(&old, &pair), [(10.0, 70.0), (10.0, 50.0)]);
    old.apply_to(&mut host).unwrap();
    let before_hits = host.hits[cache_leaf as usize].get();
    let warm = compute(&host, &[&pair, &old_limiter], &[]);
    assert_eq!(x_ranges(&warm, &pair), [(10.0, 70.0), (10.0, 50.0)]);
    assert!(
        host.hits[cache_leaf as usize].get() > before_hits,
        "an actual eligible independent leaf supplies a real warm CacheOf answer"
    );
    warm.apply_to(&mut host).unwrap();

    // Replace one actual captured text source and bind its newly issued origin.
    // Reuse the genuine native text node rather than inventing a prepared handle.
    let old_text_node = old_limiter.binding.origins[0].node;
    host.children[limiter as usize].clear();
    let mut new_limiter = Paragraph::new(
        &mut host,
        limiter,
        &[Part::Text("DDDDDDDDDD")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let new_text_node = new_limiter.binding.origins[0].node;
    host.inputs[old_text_node as usize] = host.inputs[new_text_node as usize].clone();
    host.children[limiter as usize] = vec![old_text_node];
    new_limiter.binding.nodes = vec![old_text_node];
    new_limiter.binding.origins[0].node = old_text_node;
    let changed = compute(&host, &[&pair, &new_limiter], &[limiter, old_text_node]);
    assert_eq!(x_ranges(&changed, &pair), [(0.0, 60.0), (0.0, 40.0)]);
    assert!(changed.invalidated_nodes().contains(&0));
    changed.apply_to(&mut host).unwrap();
    let replay = compute(&host, &[&pair, &new_limiter], &[]);
    assert_eq!(x_ranges(&replay, &pair), [(0.0, 60.0), (0.0, 40.0)]);
    assert_eq!(placed(&changed, &pair).key(), placed(&replay, &pair).key());
    assert_eq!(
        placed(&old, &old_limiter).source().text().code_units(),
        "DDDDDDDD".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(x_ranges(&old, &pair), [(10.0, 70.0), (10.0, 50.0)]);
    assert_eq!(
        placed(&old, &pair)
            .cursor(t::Cursor::new(0, t::Affinity::After))
            .unwrap()
            .rect()
            .origin
            .x,
        10.0
    );
    assert_eq!(
        placed(&changed, &pair)
            .cursor(t::Cursor::new(0, t::Affinity::After))
            .unwrap()
            .rect()
            .origin
            .x,
        0.0
    );
    assert_ne!(placed(&old, &pair).key(), placed(&changed, &pair).key());

    let mut cold_host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let a = cold_host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let b = cold_host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    let leaf = cold_host.child_block(0, l::Direction::Ltr, l::TextGroupAlign::None);
    cold_host.box_change(leaf, |node| {
        node.block_flow_context = l::BlockFlowContext::Independent;
        node.size.height =
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(5.0)).unwrap());
    });
    let cold_pair = Paragraph::pair(&mut cold_host, a);
    let cold_limiter = Paragraph::new(
        &mut cold_host,
        b,
        &[Part::Text("DDDDDDDDDD")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let cold = compute(&cold_host, &[&cold_pair, &cold_limiter], &[]);
    assert_eq!(x_ranges(&cold, &cold_pair), x_ranges(&changed, &pair));
    assert_eq!(
        cold.final_entries()
            .iter()
            .map(|row| (row.node(), row.output()))
            .collect::<Vec<_>>(),
        changed
            .final_entries()
            .iter()
            .map(|row| (row.node(), row.output()))
            .collect::<Vec<_>>()
    );
}
#[test]
fn changed_limiting_text_recomputes_descendant_peers_through_real_cache_and_preserves_old_batches()
{
    changed_limiter::<f32>();
    changed_limiter::<f64>();
}
