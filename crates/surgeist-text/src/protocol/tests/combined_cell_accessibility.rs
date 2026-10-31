//! Original-source Cell accessibility and placed contribution bounds.

#![cfg(feature = "text-accessibility")]

use super::*;
use crate::{Accessibility, Cursor, Selection};
use accesskit::{Node, NodeId, Role, TextPosition, TextSelection, TreeUpdate};

// Follow the published logical child inventory, not TreeUpdate.nodes order.
fn carriers(update: &TreeUpdate, parent: NodeId) -> Vec<(NodeId, &Node)> {
    let (_, paragraph) = update.nodes.iter().find(|(id, _)| *id == parent).unwrap();
    assert_eq!(paragraph.role(), Role::Paragraph);
    paragraph
        .children()
        .iter()
        .map(|id| {
            let (_, node) = update
                .nodes
                .iter()
                .find(|(candidate, _)| candidate == id)
                .unwrap();
            assert_eq!(node.role(), Role::TextRun);
            (*id, node)
        })
        .collect()
}

// AccessKit character_lengths are UTF-8 lengths of selectable units. Decoding
// those facts permits one carrier with [1,1] or two carriers with [1] apiece;
// neither the number of TextRun nodes nor physical caret edges defines a unit.
fn original_units(update: &TreeUpdate, parent: NodeId) -> Vec<(&str, TextPosition, TextPosition)> {
    let mut units = Vec::new();
    for (id, node) in carriers(update, parent) {
        let value = node.value().expect("original TextRun value");
        let mut byte = 0;
        for (character_index, &length) in node.character_lengths().iter().enumerate() {
            let end = byte + usize::from(length);
            assert!(byte < end && end <= value.len());
            assert!(value.is_char_boundary(byte) && value.is_char_boundary(end));
            units.push((
                &value[byte..end],
                TextPosition {
                    node: id,
                    character_index,
                },
                TextPosition {
                    node: id,
                    character_index: character_index + 1,
                },
            ));
            byte = end;
        }
        assert_eq!(
            byte,
            value.len(),
            "lengths cover the original UTF-8 value exactly"
        );
    }
    units
}

#[test]
fn placed_aa_cell_enriches_original_ax_units_without_turning_them_into_physical_carets() {
    // The maintained fixture already qualifies the exact native Ahem derivative,
    // all three authored-off width requests, raw A35 advances10/10, H metrics8/-2,
    // one-em10 Cell, actual .5 horizontal frame, and original single-origin AA.
    let (layout, origin) = super::ranges::fixture();
    let analyzed = layout.analyzed_source();
    assert_eq!(origin.view().aggregate_range(), Range::new(0, 2));
    assert_eq!(origin.view().source(), layout.source());
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), 2);
    assert_eq!(cell.used_unit_count(), 2);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x41]);
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x41]
    );

    // Physical Cell interaction remains one opaque unit with only two issued
    // outer edges. These do NOT authorize replacing the original AX partition.
    let positions = layout.positions().collect::<Vec<_>>();
    assert_eq!(positions.len(), 2);
    assert_eq!(
        positions
            .iter()
            .map(|p| p.source_cursor())
            .collect::<Vec<_>>(),
        [
            Cursor::new(0, Affinity::After),
            Cursor::new(2, Affinity::Before),
        ]
    );
    assert_eq!(
        positions
            .iter()
            .map(|p| layout.position_geometry(p).unwrap().rect())
            .collect::<Vec<_>>(),
        [
            Rect::new(95.0, 20.0, 10.0, 1.0),
            Rect::new(95.0, 30.0, 10.0, 1.0),
        ]
    );
    assert_eq!(layout.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
    let packets = layout.glyph_runs();
    let [packet] = packets.as_slice() else {
        panic!("one actual normal-AA packet")
    };
    assert_eq!(packet.font_size(), 10.0);
    let frame = packet.glyph_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.outline_x(), Point::new(0.5, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));

    let parent = NodeId(1);
    let mut state = Accessibility::new(70_000);
    let source = analyzed
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    let source_units = original_units(&source, parent);
    assert_eq!(
        source_units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["A", "A"]
    );
    for (_, node) in carriers(&source, parent) {
        assert_eq!(node.bounds(), None);
        assert_eq!(node.character_positions(), None);
        assert_eq!(node.character_widths(), None);
        assert_eq!(node.font_family(), None);
        assert_eq!(node.font_size(), None);
    }
    // Host index1 has two original incident roles even though neither is an
    // additional physical Cell caret. Their logical AX positions remain exact.
    for (cursor, expected) in [
        (Cursor::new(0, Affinity::After), source_units[0].1),
        (Cursor::new(1, Affinity::Before), source_units[0].2),
        (Cursor::new(1, Affinity::After), source_units[1].1),
        (Cursor::new(2, Affinity::Before), source_units[1].2),
    ] {
        assert_eq!(
            analyzed.access_position(&state, cursor).unwrap(),
            Some(expected)
        );
    }
    for (index, unit) in source_units.iter().enumerate() {
        let start = Cursor::new(index, Affinity::After);
        let end = Cursor::new(index + 1, Affinity::Before);
        assert_eq!(
            analyzed
                .access_selection(&state, Selection::new(start, end))
                .unwrap(),
            Some(TextSelection {
                anchor: unit.1,
                focus: unit.2
            })
        );
        assert_eq!(
            analyzed
                .access_selection(&state, Selection::new(end, start))
                .unwrap(),
            Some(TextSelection {
                anchor: unit.2,
                focus: unit.1
            })
        );
    }

    // Placement enriches the same original units without changing logical selection.
    let ax_origin = Point::new(3.0, 7.0);
    let placed = layout.accessibility_update(&mut state, parent, parent, ax_origin)
        .expect("placed Cell AX must preserve original AA units and enrich them with actual inner bounds");
    let placed_units = original_units(&placed, parent);
    assert_eq!(
        placed_units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["A", "A"]
    );

    // Independent coordinates: raw horizontal edges0/10/20 become95/100/105;
    // font edges-8/+2 at baseline28 become20/30. AX origin3/7 is applied once.
    // For a valid grouped carrier, its bounds are the union of ONLY its actual
    // original units. Two carriers must expose the two separate5x10 rectangles;
    // a single [1,1] carrier may expose their actual10x10 union.
    let mut first = 0;
    for (_, node) in carriers(&placed, parent) {
        let end = first + node.character_lengths().len();
        assert!(end <= 2);
        assert_eq!(
            node.bounds(),
            Some(accesskit::Rect {
                x0: 98.0 + 5.0 * first as f64,
                y0: 27.0,
                x1: 98.0 + 5.0 * end as f64,
                y1: 37.0,
            })
        );
        assert_eq!(
            node.character_positions(),
            None,
            "Cell logical units are not physical caret stops"
        );
        assert_eq!(
            node.character_widths(),
            None,
            "bounds do not authorize independent Cell caret advances"
        );
        first = end;
    }
    assert_eq!(first, 2);
    for (cursor, expected) in [
        (Cursor::new(0, Affinity::After), placed_units[0].1),
        (Cursor::new(1, Affinity::Before), placed_units[0].2),
        (Cursor::new(1, Affinity::After), placed_units[1].1),
        (Cursor::new(2, Affinity::Before), placed_units[1].2),
    ] {
        assert_eq!(
            layout.access_position(&state, cursor).unwrap(),
            Some(expected)
        );
    }
    for (index, unit) in placed_units.iter().enumerate() {
        let start = Cursor::new(index, Affinity::After);
        let end = Cursor::new(index + 1, Affinity::Before);
        assert_eq!(
            layout
                .access_selection(&state, Selection::new(start, end))
                .unwrap(),
            Some(TextSelection {
                anchor: unit.1,
                focus: unit.2
            })
        );
        assert_eq!(
            layout
                .access_selection(&state, Selection::new(end, start))
                .unwrap(),
            Some(TextSelection {
                anchor: unit.2,
                focus: unit.1
            })
        );
    }
}

#[test]
fn source_only_all_a_space_a_keeps_three_original_units_in_one_cell_word_span() {
    // Pure Source admission/analysis: no System, fonts, preparation or placement.
    let (processed, analyzed) = super::super::captured("A A", WhiteSpace::Preserve, 0.0);
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), 3);
    assert_eq!(cell.used_unit_count(), 3);
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        [0x41, 0x20, 0x41]
    );
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x20, 0x41]
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 3);
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.is_used());
    }

    let parent = NodeId(1);
    let mut state = Accessibility::new(80_000);
    let update = analyzed
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    let units = original_units(&update, parent);
    assert_eq!(
        units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["A", " ", "A"]
    );
    let mut first = 0;
    for (_, node) in carriers(&update, parent) {
        assert_eq!(node.bounds(), None);
        assert_eq!(node.character_positions(), None);
        assert_eq!(node.character_widths(), None);
        assert_eq!(node.font_family(), None);
        assert_eq!(node.font_size(), None);
        // AccessKit0.24.1 defines the document/paragraph first word start
        // implicitly (lib.rs2051–2054). Explicit0 there is optional. Elsewhere
        // an empty list means this carrier contains only a word's middle/end.
        for &local_start in node.word_starts() {
            assert!(usize::from(local_start) < node.character_lengths().len());
            assert_eq!(
                first + usize::from(local_start),
                0,
                "the interior SPACE and final A do not start another Cell word"
            );
        }
        first += node.character_lengths().len();
    }
    assert_eq!(first, 3);
}
