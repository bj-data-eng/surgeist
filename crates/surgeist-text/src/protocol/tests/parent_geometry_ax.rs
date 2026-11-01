//! Original AX units retain the geometry of parent-side relocated glyphs.
//! Nested captured owners qualify physical bounds and ambiguous source cursors.
use super::*;
use accesskit::{NodeId, Role, TextPosition, TextSelection};

#[test]
fn placed_box_edges_project_relocated_original_units_to_ax() {
    // This maintained factory authentically captures A in P/C at source0..1,
    // then outside BB at1..3, with a real Continuous original connection.
    let prepared = nested_capture("ABB");
    assert_eq!(
        prepared.analyzed_source().source().id(),
        Some(Id::from_u64(1_507_100))
    );
    assert_eq!(
        prepared.analyzed_source().source().text(),
        &crate::Text::from("ABB")
    );
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    let controls = prepared
        .control_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    let [ps, cs, ce, pe] = controls.as_slice() else {
        panic!("actual four event-ordered endpoints")
    };
    let p = ps.captured_control().unwrap().scope();
    let c = cs.captured_control().unwrap().scope();
    assert_eq!(c.parent(), Some(p.clone()));
    let domain = p.parent().unwrap();
    assert_eq!(domain.role(), crate::ScopeRole::Domain);
    assert_eq!(p.entry_control(), ps.captured_control());
    assert_eq!(p.exit_control(), pe.captured_control());
    assert_eq!(c.entry_control(), cs.captured_control());
    assert_eq!(c.exit_control(), ce.captured_control());
    let p_context = prepared
        .inline_box_edge_context(
            crate::InlineBoxSource::captured(p.clone()),
            crate::InlineBoxSource::captured(domain),
            crate::LineEdgeSide::Start,
        )
        .unwrap();
    let c_context = prepared
        .inline_box_edge_context(
            crate::InlineBoxSource::captured(c),
            crate::InlineBoxSource::captured(p),
            crate::LineEdgeSide::End,
        )
        .unwrap();
    // The physical used parent facts are B LTR for P, actual P RTL for C.
    // No Source level or bidi override is manufactured to encode those facts.
    let mut input = request(&prepared);
    input.context.spacing = SpacingPolicy::None;
    input.controls = [
        (ps, 1.0, p_context.clone()),
        (cs, 2.0, c_context.clone()),
        (ce, 3.0, c_context),
        (pe, 4.0, p_context),
    ]
    .into_iter()
    .map(|(control, advance, context)| ControlAllocation {
        occurrence: control.clone(),
        strut: None,
        boundary: Some(crate::InlineBoundaryAllocation::box_edge(context, advance).unwrap()),
    })
    .collect();
    assert!(prepared.boundary_context_demand(&input).unwrap().is_none());
    let line = prepared
        .select(input)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (40.0, 8.0, 10.0)
    );
    let actual = line
        .run_views()
        .flat_map(|run| {
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            run.glyphs()
                .map(|g| {
                    assert_eq!(
                        instance
                            .glyph_outline(g.id(), run.face().catalogue_identity())
                            .unwrap()
                            .bounds(),
                        Some(crate::Rect::new(0.0, -8.0, 10.0, 10.0))
                    );
                    (
                        g.range(),
                        g.id(),
                        run.origin().x + g.pen() + f64::from(g.x()),
                        g.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    // Pstart0..1, Cend1..4, A4..14, Cstart14..16, Pend16..20,
    // then outside B20..30 and B30..40. Raw provider advances stay10.
    assert_eq!(
        actual,
        [
            (Range::new(0, 1), 35, 4.0, 10.0),
            (Range::new(1, 2), 36, 20.0, 10.0),
            (Range::new(2, 3), 36, 30.0, 10.0)
        ]
    );
    let layout = place(prepared, line, 3.0, 12.0).unwrap();
    let mut state = crate::Accessibility::new(10);
    let parent = NodeId(1000);
    let update = layout
        .accessibility_update(&mut state, parent, parent, crate::Point::new(100.0, 200.0))
        .unwrap();
    let paragraph = &update.nodes.iter().find(|(id, _)| *id == parent).unwrap().1;
    assert_eq!(paragraph.role(), Role::Paragraph);
    // Original AX units may use any valid TextRun grouping. Decode actual UTF8
    // lengths in paragraph-child order and assert each carrier's true unit hull.
    let expected = [
        ("A", 107.0, 117.0),
        ("B", 123.0, 133.0),
        ("B", 133.0, 143.0),
    ];
    let mut units = Vec::new();
    let mut source_unit = 0;
    for id in paragraph.children() {
        let node = &update
            .nodes
            .iter()
            .find(|(actual, _)| actual == id)
            .unwrap()
            .1;
        assert_eq!(node.role(), Role::TextRun);
        let first = source_unit;
        let mut byte = 0;
        let value = node.value().expect("retained original UTF8 value");
        for (index, length) in node.character_lengths().iter().copied().enumerate() {
            let end = byte + usize::from(length);
            assert_eq!(&value[byte..end], expected[source_unit].0);
            units.push((
                TextPosition {
                    node: *id,
                    character_index: index,
                },
                TextPosition {
                    node: *id,
                    character_index: index + 1,
                },
            ));
            byte = end;
            source_unit += 1;
        }
        assert_eq!(byte, value.len());
        assert!(first < source_unit);
        let bounds = node.bounds().expect("actual placed original contributions");
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (expected[first].1, 204.0, expected[source_unit - 1].2, 214.0)
        );
    }
    assert_eq!(source_unit, 3);
    // Distinct source-less endpoint occurrences share these raw offsets.
    // A source Cursor cannot choose their actual event identity.
    for (index, endpoints) in [(0, [ps, cs]), (1, [ce, pe])] {
        assert_ne!(endpoints[0], endpoints[1]);
        for endpoint in endpoints {
            assert_eq!(endpoint.view().source_position().get(), index);
        }
        for affinity in [Affinity::Before, Affinity::After] {
            let error = layout
                .access_position(&state, crate::Cursor::new(index, affinity))
                .expect_err("a source Cursor cannot choose distinct captured endpoint occurrences");
            assert_eq!(
                error.detail(),
                Some(&crate::ErrorDetail::InvalidTextProtocol {
                    reason: ProtocolError::AmbiguousSourcePosition { index, affinity },
                })
            );
        }
    }
    // The ordinary final B has no competing control occurrence at either end.
    assert_eq!(units.len(), 3);
    let final_start = crate::Cursor::new(2, Affinity::After);
    let final_end = crate::Cursor::new(3, Affinity::Before);
    assert_eq!(
        layout
            .access_position(&state, crate::Cursor::new(2, Affinity::Before))
            .unwrap(),
        Some(units[1].1)
    );
    assert_eq!(
        layout.access_position(&state, final_start).unwrap(),
        Some(units[2].0)
    );
    assert_eq!(
        layout.access_position(&state, final_end).unwrap(),
        Some(units[2].1)
    );
    assert_eq!(
        layout
            .access_selection(&state, crate::Selection::new(final_start, final_end))
            .unwrap(),
        Some(TextSelection {
            anchor: units[2].0,
            focus: units[2].1
        })
    );
    assert_eq!(
        layout
            .access_selection(&state, crate::Selection::new(final_end, final_start))
            .unwrap(),
        Some(TextSelection {
            anchor: units[2].1,
            focus: units[2].0
        })
    );
    for (anchor, focus, index, affinity) in [
        (
            crate::Cursor::new(0, Affinity::After),
            final_end,
            0,
            Affinity::After,
        ),
        (
            final_start,
            crate::Cursor::new(1, Affinity::Before),
            1,
            Affinity::Before,
        ),
    ] {
        let error = layout
            .access_selection(&state, crate::Selection::new(anchor, focus))
            .expect_err("AX source selection preserves actual endpoint-occurrence ambiguity");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::AmbiguousSourcePosition { index, affinity },
            })
        );
    }
}
