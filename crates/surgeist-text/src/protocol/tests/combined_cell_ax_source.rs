//! Original Cell units and word carriers around genuine ignored LF.

#![cfg(feature = "text-accessibility")]

use super::*;
use crate::{Accessibility, Cursor, Selection};
use accesskit::{Node, NodeId, Role, TextPosition, TextSelection, TreeUpdate};

const PARENT: NodeId = NodeId(1);

fn carrier<'a>(update: &'a TreeUpdate, original: &str) -> (NodeId, &'a Node) {
    let (_, parent) = update.nodes.iter().find(|(id, _)| *id == PARENT).unwrap();
    assert_eq!(parent.role(), Role::Paragraph);
    // This is the adopted necessary newline carrier, not an incidental global
    // node-count convention. Splitting off an LF-ending run would introduce an
    // implicit paragraph/word boundary in the actual locked consumer0.36.
    let [id] = parent.children() else {
        panic!("one actual Cell owns the complete contiguous newline-bearing carrier");
    };
    let (_, node) = update
        .nodes
        .iter()
        .find(|(candidate, _)| candidate == id)
        .unwrap();
    assert_eq!(node.role(), Role::TextRun);
    assert_eq!(node.value(), Some(original));
    let lengths = original
        .chars()
        .map(|ch| u8::try_from(ch.len_utf8()).unwrap())
        .collect::<Vec<_>>();
    // These particular originals are A, one or more LF Controls, then acute.
    // GB4/GB5 break before/after every LF: all original scalar boundaries are
    // host-EGC boundaries, including the standalone final mark. They become
    // A+acute ONE EGC only after Cell use removes the genuine original LFs.
    assert_eq!(node.character_lengths(), lengths.as_slice());
    let mut offset = 0;
    for (&length, ch) in node.character_lengths().iter().zip(original.chars()) {
        let end = offset + usize::from(length);
        assert_eq!(&original[offset..end], ch.to_string());
        offset = end;
    }
    assert_eq!(offset, original.len());
    // Document/paragraph start supplies implicit0; redundant explicit0 is also
    // legal. There is no interior start at LF, acute, or a narrowed large index.
    assert!(node.word_starts().iter().all(|&index| index == 0));
    assert_eq!(node.character_positions(), None);
    assert_eq!(node.character_widths(), None);
    (*id, node)
}

fn original_cell(original: &str) -> tabs::Captured {
    let captured = super::capture(original);
    assert_eq!(captured.processed.source().text(), &Text::from(original));
    assert_eq!(captured.processed.processed_text(), &Text::from(original));
    assert!(original.starts_with("A\n") && original.ends_with('\u{301}'));
    assert!(
        original
            .chars()
            .skip(1)
            .take(original.chars().count() - 2)
            .all(|ch| ch == '\n')
    );
    let count = original.encode_utf16().count();
    let cells = captured.analyzed.combined_cells().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine captured All Cell")
    };
    assert_eq!(cell.scope(), captured.processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(captured.processed.captured_scope(0).unwrap())
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), count);
    assert_eq!(cell.used_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x301]);
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x301]
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), count);
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), captured.processed.source());
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            captured.processed.captured_origin(index).unwrap()
        );
        assert_eq!(
            piece.paragraph(),
            captured.processed.captured_paragraph(0).unwrap()
        );
        assert_eq!(piece.is_used(), index == 0 || index + 1 == count);
        assert_eq!(
            piece.has_presentation_emission(),
            index == 0 || index + 1 == count
        );
    }
    assert!(captured.processed.captured_control(0).is_none());
    assert!(captured.processed.captured_origin(count).is_none());
    drop(pieces);
    drop(cells);
    captured
}

fn real_placement(captured: &tabs::Captured) -> (PreparedText, Layout) {
    let original = crate::shaping_test_fonts::mark_ahem();
    let bytes = native_metric_font(&original);
    for tag in [
        *b"cmap", *b"GDEF", *b"GPOS", *b"glyf", *b"loca", *b"hhea", *b"hmtx",
    ] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original, tag));
    }
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(face.charmap().map('\u{301}'), Some(GlyphId::new(100)));
    assert!(face.charmap().map('\u{c1}').is_none());
    // The exact actual post-used provider input is unchanged by the number of
    // authored LFs. Real mark GPOS, not a substituted precomposed Á or tofu.
    super::actual_provider(&bytes, "A\u{301}", &[(35, 1000, 0, 0), (100, 0, -700, 400)]);
    let (prepared, layout) = super::prepared_layout(captured, &bytes);
    let raw = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.glyph_frame().outline_x(), Point::new(1.0, 0.0));
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(raw, [(35, 10.0), (100, 0.0)]);
    assert_eq!(
        layout
            .positions()
            .map(|position| position.source_cursor())
            .collect::<Vec<_>>(),
        [
            Cursor::new(0, Affinity::After),
            Cursor::new(
                captured.processed.source().text().code_units().len(),
                Affinity::Before
            ),
        ]
    );
    (prepared, layout)
}

fn logical_units(
    node: NodeId,
    count: usize,
    position: impl Fn(Cursor) -> crate::Result<Option<TextPosition>>,
    selection: impl Fn(Selection) -> crate::Result<Option<TextSelection>>,
) {
    for index in 0..count {
        let start = Cursor::new(index, Affinity::After);
        let end = Cursor::new(index + 1, Affinity::Before);
        let a = TextPosition {
            node,
            character_index: index,
        };
        let b = TextPosition {
            node,
            character_index: index + 1,
        };
        assert_eq!(position(start).unwrap(), Some(a), "original start{index}");
        assert_eq!(position(end).unwrap(), Some(b), "original end{}", index + 1);
        assert_eq!(
            selection(Selection::new(start, end)).unwrap(),
            Some(TextSelection {
                anchor: a,
                focus: b
            })
        );
        assert_eq!(
            selection(Selection::new(end, start)).unwrap(),
            Some(TextSelection {
                anchor: b,
                focus: a
            })
        );
        assert_eq!(
            selection(Selection::new(start, start)).unwrap(),
            Some(TextSelection {
                anchor: a,
                focus: a
            })
        );
    }
}

fn source_carrier(captured: &tabs::Captured, original: &str) {
    // No catalogue, font registration, preparation, or placement is needed to
    // qualify the source semantics, genuine word membership and logical units.
    let mut state = Accessibility::new(90_000);
    let update = captured
        .analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .expect("original LF units remain one actual Cell word in source-only AX");
    let (id, node) = carrier(&update, original);
    assert_eq!(node.bounds(), None);
    assert_eq!(node.font_family(), None);
    assert_eq!(node.font_size(), None);
    assert_eq!(node.text_direction(), None);
    assert_eq!(node.previous_on_line(), None);
    assert_eq!(node.next_on_line(), None);
    logical_units(
        id,
        original.encode_utf16().count(),
        |cursor| captured.analyzed.access_position(&state, cursor),
        |selection| captured.analyzed.access_selection(&state, selection),
    );
}

#[test]
fn source_ignored_lf_between_base_and_mark_keeps_three_original_ax_units_in_one_word() {
    let captured = original_cell("A\n\u{301}");
    source_carrier(&captured, "A\n\u{301}");
}

#[test]
fn placed_ignored_lf_stays_logical_without_borrowing_its_range_egc_closure_as_ax_membership() {
    let original = "A\n\u{301}";
    let captured = original_cell(original);
    let (prepared, layout) = real_placement(&captured);
    // Observe the actual AX contribution join before carrier aggregation: the
    // LF has no contributor even though a Range query closes onto A+acute.
    crate::accessibility::tests::assert_cell_original_memberships(
        &layout,
        &[Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)],
        &[true, false, true],
    );
    let origins = [
        super::issued(&prepared, &captured.processed, 0, Range::new(0, 1)),
        super::issued(&prepared, &captured.processed, 1, Range::new(1, 2)),
        super::issued(&prepared, &captured.processed, 2, Range::new(2, 3)),
    ];
    let base = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let lf = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    let mark = layout
        .source_range_fragments(&origins[2], Range::new(2, 3))
        .unwrap();
    assert_eq!(
        (base.units().len(), lf.units().len(), mark.units().len()),
        (1, 1, 1)
    );
    let a = base.units().next().unwrap();
    let removed = lf.units().next().unwrap();
    let m = mark.units().next().unwrap();
    assert!(removed.same_unit(&a) && removed.same_unit(&m));
    assert!(
        removed.pieces().len() > 0,
        "Range deliberately closes the marked EGC"
    );
    assert_eq!(
        removed
            .selected_intersections()
            .map(|piece| (piece.origin(), piece.local_range()))
            .collect::<Vec<_>>(),
        [(origins[1].clone(), Range::new(1, 2))]
    );
    assert_eq!(removed.expanded_intersections().len(), 3);
    // Those Range facts are a contrast control, not an AX bounds oracle.
    // The actual LF source piece has no presentation emission (original_cell).
    let mut state = Accessibility::new(91_000);
    let source = captured
        .analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let (source_id, _) = carrier(&source, original);
    let origin = Point::new(3.0, 7.0);
    let update = layout
        .accessibility_update(&mut state, PARENT, PARENT, origin)
        .expect("placed AX must preserve the original LF despite the retained A+acute EGC");
    let (id, node) = carrier(&update, original);
    assert_eq!(
        id, source_id,
        "carrier identity depends on source, not placement"
    );
    // Raw base advance10/mark0, H8/-2, baseline(95,28), unit scale, AX origin3/7.
    // This is the union of actual A/acute contributions, not LF's own rectangle.
    assert_eq!(
        node.bounds(),
        Some(accesskit::Rect {
            x0: 98.0,
            y0: 27.0,
            x1: 108.0,
            y1: 37.0
        })
    );
    logical_units(
        id,
        3,
        |cursor| layout.access_position(&state, cursor),
        |selection| layout.access_selection(&state, selection),
    );
    // Especially LF end2 must not collapse back to its start1 as a genuine
    // hard-line-break focus would; all physical Cell edges nevertheless stay2.
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(2, Affinity::Before))
            .unwrap(),
        Some(TextPosition {
            node: id,
            character_index: 2
        })
    );
    assert_eq!(layout.positions().count(), 2);
}

fn long_original() -> String {
    // 262 ORIGINAL selectable units, 263 UTF8 bytes, but every original unit is
    // just1 or2 bytes. No producer character-count or aggregate-byte cap applies.
    // The provider still sees exactly A+acute, so this tests indices rather than
    // a new large provider context, different width result or absent-font path.
    format!("A{}\u{301}", "\n".repeat(260))
}

#[test]
fn source_newline_cell_carrier_preserves_original_logical_indices_above_u8() {
    let original = long_original();
    assert_eq!(original.encode_utf16().count(), 262);
    assert_eq!(original.len(), 263);
    let captured = original_cell(&original);
    source_carrier(&captured, &original);
}

#[test]
fn placed_newline_cell_carrier_preserves_large_original_indices_with_the_same_two_glyph_payload() {
    let original = long_original();
    let captured = original_cell(&original);
    let (_prepared, layout) = real_placement(&captured);
    let mut state = Accessibility::new(92_000);
    let source = captured
        .analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let (source_id, _) = carrier(&source, &original);
    let update = layout
        .accessibility_update(&mut state, PARENT, PARENT, Point::new(0.0, 0.0))
        .expect("a producer-valid Cell aggregate has no arbitrary255-character cap");
    let (id, node) = carrier(&update, &original);
    assert_eq!(id, source_id);
    assert_eq!(node.character_lengths().len(), 262);
    assert_eq!(
        node.bounds(),
        Some(accesskit::Rect {
            x0: 95.0,
            y0: 20.0,
            x1: 105.0,
            y1: 30.0
        })
    );
    logical_units(
        id,
        262,
        |cursor| layout.access_position(&state, cursor),
        |selection| layout.access_selection(&state, selection),
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(257, Affinity::After))
            .unwrap(),
        Some(TextPosition {
            node: id,
            character_index: 257
        })
    );
    assert_eq!(
        layout
            .access_selection(
                &state,
                Selection::new(
                    Cursor::new(257, Affinity::After),
                    Cursor::new(258, Affinity::Before)
                )
            )
            .unwrap(),
        Some(TextSelection {
            anchor: TextPosition {
                node: id,
                character_index: 257
            },
            focus: TextPosition {
                node: id,
                character_index: 258
            },
        })
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(261, Affinity::After))
            .unwrap(),
        Some(TextPosition {
            node: id,
            character_index: 261
        })
    );
    assert_eq!(layout.positions().count(), 2);
}

#[test]
fn late_removed_lf_after_retained_prefix_keeps_exact_contribution_and_metadata_coverage() {
    let original = format!("{}\n\u{301}", "A".repeat(64));
    let captured = super::capture(&original);
    let cell = captured.analyzed.combined_cells().next().unwrap();
    assert_eq!(captured.analyzed.combined_cells().len(), 1);
    assert_eq!(
        (cell.selection_unit_count(), cell.used_unit_count()),
        (66, 64)
    );
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        "A".repeat(64)
            .chars()
            .map(u32::from)
            .chain([0x301])
            .collect::<Vec<_>>()
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 66);
    assert_eq!(pieces[64].aggregate(), Range::new(64, 65));
    assert!(!pieces[64].is_used() && !pieces[64].has_presentation_emission());
    let bytes = native_metric_font(&crate::shaping_test_fonts::mark_ahem());
    let mut expected = vec![(35, 1000, 0, 0); 64];
    expected.push((100, 0, -700, 400));
    super::actual_provider(&bytes, &format!("{}\u{301}", "A".repeat(64)), &expected);
    let (_prepared, layout) = super::prepared_layout(&captured, &bytes);
    let originals = (0..66)
        .map(|index| Range::new(index, index + 1))
        .collect::<Vec<_>>();
    let mut contributes = vec![true; 66];
    contributes[64] = false;
    crate::accessibility::tests::assert_cell_original_memberships(
        &layout,
        &originals,
        &contributes,
    );
    let mut state = Accessibility::new(93_000);
    let source = captured
        .analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let (source_id, _) = carrier(&source, &original);
    let placed = layout
        .accessibility_update(&mut state, PARENT, PARENT, Point::new(0.0, 0.0))
        .unwrap();
    let (id, node) = carrier(&placed, &original);
    assert_eq!(id, source_id);
    // Raw64x10 compressed by1/64 gives10; metric edges remain20..30.
    assert_eq!(
        node.bounds(),
        Some(accesskit::Rect {
            x0: 95.0,
            y0: 20.0,
            x1: 105.0,
            y1: 30.0
        })
    );
    assert_eq!(node.font_family(), None);
    assert_eq!(node.font_size(), None);
    assert_eq!(node.text_direction(), None);
    logical_units(
        id,
        66,
        |cursor| layout.access_position(&state, cursor),
        |selection| layout.access_selection(&state, selection),
    );
}
