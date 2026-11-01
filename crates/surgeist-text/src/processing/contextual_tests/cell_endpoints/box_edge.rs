//! P is a genuine declared native pair of root-scoped captured controls, not
//! an invented captured Inline. Captured Cell owner C remains a real Inline
//! whose captured parent is root. Native P encloses C's actual source interval
//! and shares that root container. Explicit P parent basis puts C.SourceStart
//! at physical End and C.SourceEnd at Start; it does not contradict a claimed
//! captured P CSS direction or rewrite Source bidi/inner-cell direction.

use super::*;

type Snapshot = (
    f64,
    Rect,
    Vec<(PreparedControl, (f64, f64))>,
    Vec<(u32, f32, f64, Range)>,
);

fn snapshot(line: &FinalizedLine) -> Snapshot {
    assert_eq!(line.cell_views().len(), 1);
    (
        line.feedback().advance,
        line.cell_views().next().unwrap().square(),
        line.control_slots()
            .map(|slot| (slot.control(), slot.inline_bounds()))
            .collect(),
        line.run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance(), glyph.pen(), glyph.range()))
                    .collect::<Vec<_>>()
            })
            .collect(),
    )
}

fn whole_cell_geometry(
    line: &FinalizedLine,
    prepared: &PreparedText,
    scope: &CapturedScope,
    controls: &[PreparedControl],
    bytes: &[u8],
    square_inline: f64,
    bounds: &[(f64, f64)],
) {
    assert_eq!(line.prepared_text(), *prepared);
    assert_eq!(line.cell_views().len(), 1);
    assert_eq!(line.feedback().advance, 15.0); // real2+E10+3, parent contributions0
    assert_eq!(line.measurements().fit().advance(), 15.0);
    assert_eq!(line.measurements().alignment().advance(), 15.0);
    assert_eq!(
        line.measurements().max_content_contribution().advance(),
        15.0
    );
    let cell = line.cell_views().next().unwrap();
    assert_eq!(cell.source_cell().scope(), *scope);
    assert_eq!(cell.source_cell().mode(), TextCombineUpright::All);
    assert_eq!(
        cell.source_cell().used_code_points().collect::<Vec<_>>(),
        [0x41]
    );
    assert_eq!(cell.square().origin.x, square_inline);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.level().get(), 0);
    let slots = line.control_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 4);
    assert_eq!(bounds.len(), 4);
    for (occurrence, &expected) in controls.iter().zip(bounds) {
        assert_eq!(
            slots
                .iter()
                .filter(|slot| slot.control() == *occurrence)
                .count(),
            1
        );
        assert_eq!(
            slots
                .iter()
                .find(|slot| slot.control() == *occurrence)
                .unwrap()
                .inline_bounds(),
            expected
        );
    }
    let runs: Vec<_> = line
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect();
    assert_eq!(
        runs.len(),
        1,
        "one real Cell provider packet; no duplicate ordinary packet"
    );
    let run = &runs[0];
    assert_eq!(run.face().bytes(), bytes);
    assert_eq!(run.face().index(), 0);
    assert_eq!(run.font_size(), 10.0);
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(run.cell().unwrap().source_cell().scope(), *scope);
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
                glyph.range(),
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 10.0, Range::new(0, 1))]
    );
}

#[test]
fn native_parent_opposed_box_edges_reorder_the_complete_cw_cell_packet() {
    let mut value = base("A", TextCombineUpright::None);
    let body = Style {
        text_combine_upright: TextCombineUpright::All,
        ..super::super::style(WhiteSpace::Preserve, 10.0)
    };
    value.source.span(Range::new(0, 1), body.clone());
    value.origins = vec![super::super::origin(&value.source, Range::new(0, 1))];
    // ONLY C is a captured Inline. P's actual endpoint occurrences are rooted
    // in the real domain outside C, with no fake Scope or invented style token.
    value.scopes.push(inline(body, 1, 2));
    value.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        }, // P
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        }, // C
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        }, // C
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        }, // P
    ];
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        event(0),
        SourceProcessingEvent::EnterScope(1),
        event(1),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        event(2),
        SourceProcessingEvent::ExitScope(1),
        event(3),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let (processed, analyzed) = analyze(value);
    assert_eq!(processed.source().text(), &Text::from("A"));
    assert_eq!(processed.processed_text(), &Text::from("A"));
    let c = processed.captured_scope(1).unwrap();
    let root = processed.captured_scope(0).unwrap();
    assert_eq!(c.parent(), Some(root.clone()));
    assert_eq!(c.entry_control(), processed.captured_control(1));
    assert_eq!(c.exit_control(), processed.captured_control(2));
    assert!(root.entry_control().is_none() && root.exit_control().is_none());
    let ps = processed.captured_control(0).unwrap();
    let cs = processed.captured_control(1).unwrap();
    let ce = processed.captured_control(2).unwrap();
    let pe = processed.captured_control(3).unwrap();
    assert_eq!(ps.scope(), root);
    assert_eq!(pe.scope(), root);
    assert_eq!(cs.scope(), c);
    assert_eq!(ce.scope(), c);
    assert!(ps.event_position() < c.enter_position());
    assert!(c.enter_position() < cs.event_position());
    assert!(cs.event_position() < processed.captured_origin(0).unwrap());
    assert!(processed.captured_origin(0).unwrap() < ce.event_position());
    assert!(ce.event_position() < c.exit_position());
    assert!(c.exit_position() < pe.event_position());
    assert_eq!(analyzed.combined_cells().len(), 1);
    let source_cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(source_cell.scope(), c);
    assert_eq!(
        (
            source_cell.selection_unit_count(),
            source_cell.used_unit_count()
        ),
        (1, 1)
    );
    assert_eq!(source_cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    let pieces: Vec<_> = source_cell.source_pieces().collect();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].source(), processed.source());
    assert_eq!(pieces[0].local(), Range::new(0, 1));
    assert_eq!(pieces[0].aggregate(), Range::new(0, 1));
    assert_eq!(
        pieces[0].origin_position(),
        processed.captured_origin(0).unwrap()
    );

    // Maintained prepare() qualifies licensed real Ahem-derived font operands:
    // UPEM1000, cmap A35, hmtx1000 and hhea800/-200. CW is an existing supported
    // profile; A10 fits the actual E10 Cell, not a fake in-flow atomic.
    assert_eq!(ORIENTATION, LineOrientation::SidewaysClockwise);
    let (_system, prepared, bytes) = prepare(&analyzed);
    assert_eq!(prepared.cell_views().len(), 1);
    assert_eq!(prepared.atomic_views().count(), 0);
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    assert_eq!(cell.level().get(), 0);
    assert_eq!(cell.source_cell().scope(), c);
    let cell_handle = cell.handle();
    let controls: Vec<_> = prepared.control_views().map(|row| row.handle()).collect();
    assert_eq!(controls.len(), 4);
    for (control, at) in controls.iter().zip([0, 0, 1, 1]) {
        assert_eq!(control.view().source_position().get(), at);
    }
    assert_eq!(controls[0].captured_control(), Some(ps.clone()));
    assert_eq!(controls[1].captured_control(), Some(cs.clone()));
    assert_eq!(controls[2].captured_control(), Some(ce.clone()));
    assert_eq!(controls[3].captured_control(), Some(pe.clone()));
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [
            PreparedParticipant::Control(controls[0].clone()),
            PreparedParticipant::Control(controls[1].clone()),
            PreparedParticipant::Cell(cell_handle),
            PreparedParticipant::Control(controls[2].clone()),
            PreparedParticipant::Control(controls[3].clone()),
        ],
        "all four actual controls are outside the COMPLETE Cell"
    );
    let source_levels: Vec<_> = prepared
        .control_views()
        .map(|row| (row.handle(), row.level()))
        .collect();

    let ordinary = finish(&prepared, &[0.0, 2.0, 3.0, 0.0]);
    whole_cell_geometry(
        &ordinary,
        &prepared,
        &c,
        &controls,
        &bytes,
        2.0,
        &[(0.0, 0.0), (0.0, 2.0), (12.0, 15.0), (15.0, 15.0)],
    );
    let old = snapshot(&ordinary);
    let parent = prepared.native_inline_box(ps, pe).unwrap();
    assert!(
        parent.captured_scope().is_none(),
        "P is a real declared native pair, NOT root-as-an-Inline"
    );
    let owner = prepared.native_inline_box(cs, ce).unwrap();
    assert_eq!(owner, InlineBoxSource::captured(c.clone()));
    let context = prepared
        .inline_box_edge_context(owner, parent.clone(), LineEdgeSide::End)
        .unwrap();
    assert_eq!(context.parent(), &parent);
    assert_eq!(context.start_side(), LineEdgeSide::End);
    // This is the normalized physical basis of actual native P. C's captured
    // parent remains root; P's endpoints have that SAME real root scope and
    // enclose C in event order. No captured parent direction is contradicted.
    let mut request = ordinary.selected_line().request().clone();
    for allocation in &mut request.controls {
        if allocation.occurrence == controls[1] {
            allocation.boundary =
                Some(InlineBoundaryAllocation::box_edge(context.clone(), 2.0).unwrap());
        } else if allocation.occurrence == controls[2] {
            allocation.boundary =
                Some(InlineBoundaryAllocation::box_edge(context.clone(), 3.0).unwrap());
        } else {
            assert_eq!(allocation.boundary.as_ref().unwrap().advance(), 0.0);
        }
    }
    let selected = prepared.select(request).unwrap();
    assert_ne!(selected.request(), ordinary.selected_line().request());
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    // Independent opposed basis: C.End3 on the LEFT, one E10 packet, then
    // C.Start2 on the RIGHT. A Cell-only bypass to the ordinary2..12 ordering
    // cannot satisfy this3..13 square or the distinct actual endpoint slots.
    whole_cell_geometry(
        &line,
        &prepared,
        &c,
        &controls,
        &bytes,
        3.0,
        &[(0.0, 0.0), (13.0, 15.0), (0.0, 3.0), (15.0, 15.0)],
    );
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(snapshot(&replay), snapshot(&line));
    assert_eq!(
        replay.selected_line().request(),
        line.selected_line().request()
    );
    assert_eq!(snapshot(&ordinary), old);
    assert_eq!(
        prepared
            .control_views()
            .map(|row| (row.handle(), row.level()))
            .collect::<Vec<_>>(),
        source_levels
    );

    // Real CW placement, same input anchor for each immutable result. Actual
    // horizontal hhea8/-2 gives cross-left100-2=98 and local baseline8. Thus
    // ordinary flow top20+2 yields glyph baseline30; BoxEdge flow top20+3
    // yields31. Its inner axes/scale/resource/pens remain identical.
    for (actual, expected) in [
        (&ordinary, Point::new(98.0, 30.0)),
        (&line, Point::new(98.0, 31.0)),
        (&replay, Point::new(98.0, 31.0)),
    ] {
        let layout = Layout::from_finalized_lines(PlacementRequest {
            prepared: prepared.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![PlacementEntry::Formatted(LinePlacement {
                line: actual.clone(),
                anchor: LineAnchor {
                    inline_origin: 20.0,
                    baseline: 100.0,
                },
                orientation: ORIENTATION,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(layout.source(), processed.source());
        assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
        assert_eq!(layout.lines().len(), 1);
        let runs: Vec<_> = layout
            .run_views()
            .filter(|run| run.glyphs().next().is_some())
            .collect();
        assert_eq!(runs.len(), 1);
        let run = &runs[0];
        assert_eq!(run.face().bytes(), bytes.as_slice());
        assert_eq!(run.orientation(), ORIENTATION);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        let frame = run.glyph_frame();
        assert_eq!(frame.origin(), expected);
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        assert_eq!(
            run.glyphs()
                .map(|glyph| (
                    glyph.id(),
                    glyph.pen(),
                    glyph.x(),
                    glyph.y(),
                    glyph.advance(),
                ))
                .collect::<Vec<_>>(),
            [(35, 0.0, 0.0, 0.0, 10.0)]
        );
    }
    assert_eq!(snapshot(&ordinary), old);
}
