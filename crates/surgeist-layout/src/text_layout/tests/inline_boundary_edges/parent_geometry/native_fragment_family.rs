//! A submitted native C remains distinct from its actual captured parent P.
use super::*;

// The four codepoint-free markers are L1-transparent here and A is strong L.
// Use their authentic original levels with the independent five-row L2 rule;
// an End label alone is not an ordinary Contribution's physical-side promise.
fn ordinary_zero_strips(fixture: &Fixture) -> [(u32, f64); 4] {
    let level = |node| control(fixture, node).view().level().get();
    let levels = [
        level(20),
        level(22),
        fixture
            .binding
            .prepared
            .unit_views()
            .next()
            .unwrap()
            .level()
            .get(),
        level(23),
        level(21),
    ];
    let mut order = [0usize, 1, 2, 3, 4];
    if let Some(lowest) = levels.iter().copied().filter(|level| level & 1 != 0).min() {
        for threshold in (lowest..=*levels.iter().max().unwrap()).rev() {
            let mut cursor = 0;
            while cursor < order.len() {
                if levels[order[cursor]] < threshold {
                    cursor += 1;
                    continue;
                }
                let first = cursor;
                while cursor < order.len() && levels[order[cursor]] >= threshold {
                    cursor += 1;
                }
                order[first..cursor].reverse();
            }
        }
    }
    let mut positions = [0.0; 5];
    let mut pen = 0.0;
    for row in order {
        positions[row] = pen;
        if row == 2 {
            pen += 10.0;
        }
    }
    [
        (20, positions[0]),
        (22, positions[1]),
        (23, positions[3]),
        (21, positions[4]),
    ]
}

fn lane<S: LayoutScalar>(rtl: bool, box_edges: bool) {
    // This maintained fixture has real captured P nominations0/1 and genuine
    // additional C controls2/3 inside P, with no invented captured C scope.
    let mut fixture = Fixture::nested_parent("A", rtl, false, false);
    source_premises(&fixture, "A", rtl, false);
    let parent_scope = control(&fixture, 20).captured_control().unwrap().scope();
    assert_eq!(
        parent_scope.domain().domain_topology(),
        text::SourceProcessingTopology::Complete
    );
    let parent_owner = text::InlineBoxSource::captured(parent_scope);
    let native_owner = fixture
        .binding
        .prepared
        .native_inline_box(
            control(&fixture, 22).captured_control().unwrap(),
            control(&fixture, 23).captured_control().unwrap(),
        )
        .unwrap();
    assert!(native_owner.captured_scope().is_none());
    assert_ne!(native_owner, parent_owner);
    let units = fixture.binding.prepared.unit_views().collect::<Vec<_>>();
    assert_eq!(units.len(), 1);
    let text_participant = text::PreparedParticipant::Text(units[0].handle());
    let original_levels = fixture
        .binding
        .prepared
        .control_views()
        .map(|view| (view.handle(), view.level().get()))
        .collect::<Vec<_>>();
    let width = if box_edges { 20.0 } else { 10.0 };
    let prior = compute::<S>(&fixture, rtl, width, false, box_edges);
    assert!(prior.placed_text()[0].inline_box_fragments().is_none());

    fixture.binding.inline_box_fragments = crate::InlineBoxFragmentationDemand::CompleteBodies;
    // compute uses the actual checked LayoutFlowInput constructor, the genuine
    // NativePair22/23 inventory and the exact same maintained native tree.
    let batch = compute::<S>(&fixture, rtl, width, false, box_edges);
    assert_eq!(batch.placed_text().len(), 1);
    let placed = &batch.placed_text()[0];
    let view = placed.inline_box_fragments().unwrap();
    assert_eq!(view.family().prepared_text(), fixture.binding.prepared);
    let lines = finalized_lines(&batch);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert_eq!(line.inline_box_family(), Some(view.family()));
    let selected = line.selected_line();
    assert_eq!(selected.inline_box_family(), Some(view.family()));
    let selected_piece = selected
        .inline_box_pieces()
        .unwrap()
        .find(|piece| piece.owner() == native_owner)
        .unwrap();
    let finalized_piece = line
        .inline_box_pieces()
        .unwrap()
        .find(|piece| piece.owner() == native_owner)
        .unwrap();
    let placed_piece = view
        .pieces()
        .find(|piece| piece.fragment().owner() == native_owner)
        .unwrap();
    assert_eq!(selected_piece.fragment(), finalized_piece.fragment());
    assert_eq!(placed_piece.fragment(), finalized_piece.fragment());
    assert_eq!(
        placed_piece.finalized_piece().fragment(),
        finalized_piece.fragment()
    );
    let parent = finalized_piece.fragment().parent().unwrap();
    assert_eq!(parent.owner(), parent_owner);
    assert_eq!(
        finalized_piece
            .fragment()
            .nearest_common_ancestor(&parent)
            .unwrap(),
        parent
    );
    assert!(
        line.inline_box_pieces()
            .unwrap()
            .any(|piece| piece.fragment() == parent)
    );
    let members = finalized_piece
        .members()
        .filter_map(|member| match member.participant() {
            Some(participant @ text::PreparedParticipant::Text(_)) => {
                Some((participant.clone(), member.unit_source_range()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(members, [(text_participant, text::Range::new(0, 1))]);

    // Literal Ahem and endpoint expectations are the maintained nested case's
    // oracles, independent of the new family projection/readback.
    let glyph_x = if !box_edges {
        0.0
    } else if rtl {
        6.0
    } else {
        4.0
    };
    assert_eq!(line.feedback().advance, width);
    assert_eq!(glyphs(line), [(35, glyph_x, 10.0)]);
    if box_edges {
        let strips = if rtl {
            [
                (21, 0.0, 4.0, false),
                (22, 4.0, 6.0, false),
                (23, 16.0, 19.0, true),
                (20, 19.0, 20.0, true),
            ]
        } else {
            [
                (20, 0.0, 1.0, false),
                (23, 1.0, 4.0, false),
                (22, 14.0, 16.0, true),
                (21, 16.0, 20.0, true),
            ]
        };
        for (node, start, end, right) in strips {
            assert_strip(&batch, &fixture, node, start, end, right);
        }
    } else {
        // P is genuinely undecorated and all endpoints remain Contributions.
        // A demanded identity must not promote them into BoxEdge operations.
        let prior_line = finalized_lines(&prior).remove(0);
        for (node, position) in ordinary_zero_strips(&fixture) {
            let occurrence = control(&fixture, node);
            let prior_strip = prior_line
                .control_slots()
                .find(|slot| slot.control() == occurrence)
                .unwrap()
                .strip_interval();
            let actual = line
                .control_slots()
                .find(|slot| slot.control() == occurrence)
                .unwrap()
                .strip_interval();
            assert_eq!(prior_strip, (position, position));
            assert_eq!(
                actual, prior_strip,
                "family metadata must preserve Contribution node{node}"
            );
        }
    }
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.inline_box_family(), Some(view.family()));
    assert_eq!(
        replay
            .inline_box_pieces()
            .unwrap()
            .find(|piece| piece.owner() == native_owner)
            .unwrap()
            .fragment(),
        finalized_piece.fragment()
    );
    assert_eq!(glyphs(&replay), [(35, glyph_x, 10.0)]);
    assert_eq!(
        fixture
            .binding
            .prepared
            .control_views()
            .map(|row| (row.handle(), row.level().get()))
            .collect::<Vec<_>>(),
        original_levels
    );
    assert_eq!(glyphs(&finalized_lines(&prior)[0]), [(35, glyph_x, 10.0)]);
}

#[test]
fn demanded_actual_native_pair_retains_owner_parent_and_existing_geometry() {
    for rtl in [false, true] {
        for box_edges in [false, true] {
            lane::<f32>(rtl, box_edges);
            lane::<f64>(rtl, box_edges);
        }
    }
}
