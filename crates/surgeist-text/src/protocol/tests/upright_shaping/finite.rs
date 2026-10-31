//! Finite native upright alignment consumes one real SPACE allocation and keeps
//! the outer line frame separate from physical content and editing geometry.
use super::*;
use crate::{
    Affinity, Alignment, Cursor, GlyphBasis, Hit, LineAlignmentInsets, Selection,
    SpacingSlotDisposition, SpacingSlotOwner,
};

#[test]
fn finite_upright_centering_applies_indent_insets_and_actual_space_allocation_once() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x20, 3), (0x58, 58)]);
    // The native font supplies 1200 design units for X and SPACE. At size10,
    // each natural inline advance is12. The empty SPACE's nominal yMax is0.
    let face = skrifa::FontRef::new(&bytes).unwrap();
    let head = sfnt::table(&bytes, *b"head");
    assert_eq!(sfnt::u16_at(head, 50), 0);
    let loca = sfnt::table(&bytes, *b"loca");
    assert_eq!(
        (sfnt::u16_at(loca, 3 * 2) * 2, sfnt::u16_at(loca, 4 * 2) * 2),
        (40, 40),
        "equal short-loca offsets prove actual empty glyph3 without a glyf header"
    );
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    // Skrifa returns a present zero box for a valid empty glyph; None denotes
    // invalid/missing data, rather than this independently proved empty record.
    let space_bounds = metrics.bounds(GlyphId::new(3)).unwrap();
    assert_eq!(
        [
            space_bounds.x_min,
            space_bounds.y_min,
            space_bounds.x_max,
            space_bounds.y_max
        ],
        [0.0, 0.0, 0.0, 0.0]
    );
    let vertical = face.vmtx().unwrap();
    for glyph in [3, 58] {
        assert_eq!(
            (
                vertical.v_metrics()[glyph].advance.get(),
                vertical.v_metrics()[glyph].side_bearing.get()
            ),
            (1200, 100)
        );
    }
    let mut system = system_with(&bytes);
    let (prepared, natural) = upright_line(&mut system, "X X", &[r#""vert" 0"#]);
    assert_eq!(natural.feedback().advance, 36.0);
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [88, 32, 88]
    );
    let origin = prepared.origin_views().next().unwrap().handle();
    let insets = LineAlignmentInsets::try_new(2.0, 4.0).unwrap();
    let mut input = request(&prepared);
    input.context.orientation = LineOrientation::Vertical;
    input.context.base_direction = ParagraphBaseDirection::LeftToRight;
    input.context.line_box_direction = LineBoxDirection::LeftToRight;
    input.context.available_inline = Some(100.0);
    input.context.indent = 10.0;
    input.context.alignment_insets = insets;
    input.context.alignment = Alignment::Center;
    input.context.spacing = SpacingPolicy::Spaces;
    let selected = prepared.select(input).unwrap();
    let request_context = &selected.request().context;
    assert_eq!(request_context.available_inline, Some(100.0));
    assert_eq!(request_context.indent, 10.0);
    assert_eq!(request_context.alignment, Alignment::Center);
    assert_eq!(request_context.alignment_insets, insets);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let space = prepared
        .unit_views()
        .find(|unit| unit.source_range() == Range::new(1, 2))
        .unwrap()
        .handle();
    assert_eq!(
        slots[0].participants().cloned().collect::<Vec<_>>(),
        [PreparedParticipant::Text(space.clone())]
    );
    assert_eq!(
        slots[0]
            .pieces()
            .map(|piece| piece.owner())
            .collect::<Vec<_>>(),
        [SpacingSlotOwner::Source {
            origin: prepared.origin_views().next().unwrap().handle(),
            local: Range::new(1, 2),
            aggregate: Range::new(1, 2),
        }]
    );
    assert_eq!(space.prepared_text(), prepared);
    assert_eq!(space.view().source_range(), Range::new(1, 2));
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: slots[0].clone(),
                amount: 8.0,
            }]),
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.selected_line(), selected);
    assert_eq!(line.feedback().advance, 44.0);
    assert_eq!(line.measurements().fit().advance(), 44.0);
    assert_eq!(line.measurements().alignment().advance(), 44.0);
    let feedback = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(feedback.len(), 1);
    assert_eq!(feedback[0].slot, slots[0]);
    assert_eq!(feedback[0].advance, 20.0);
    assert_eq!(
        feedback[0].disposition,
        SpacingSlotDisposition::Retained { trailing: false }
    );
    // Width100 minus insets2/4 and indent10 leaves84. Advance44 leaves40;
    // centering adds20, giving line-local inline origin2+10+20=32.
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
                let inline = run.origin().x;
                run.glyphs()
                    .map(move |glyph| {
                        (
                            glyph.range(),
                            inline + glyph.pen(),
                            glyph.x(),
                            glyph.y(),
                            glyph.advance(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 32.0, -6.0, 9.0, 12.0),
            (Range::new(1, 2), 44.0, -6.0, 1.0, 12.0),
            (Range::new(2, 3), 64.0, -6.0, 9.0, 12.0),
        ]
    );
    let layout = place_upright(prepared.clone(), line);
    // The supplied inline20/cross100 anchor places the unchanged outer100
    // span at y20..120. Allocated content instead occupies y52..96.
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(94.0, 20.0, 12.0, 100.0)
    );
    assert_eq!(layout.bounds(), Rect::new(94.0, 20.0, 12.0, 100.0));
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        Point::new(100.0, 52.0)
    );
    assert!(
        layout
            .run_views()
            .all(|run| run.glyph_basis() == GlyphBasis::Unrotated)
    );
    assert!(
        layout
            .glyph_runs()
            .iter()
            .all(|run| run.glyph_basis() == GlyphBasis::Unrotated)
    );
    let second = Cursor::new(2, Affinity::After);
    assert_eq!(
        layout.cursor(second).unwrap().rect(),
        Rect::new(94.0, 84.0, 12.0, 1.0)
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(3, Affinity::Before))
            .unwrap()
            .rect(),
        Rect::new(94.0, 96.0, 12.0, 1.0)
    );
    let selection = layout
        .selection(Selection::new(
            Cursor::new(0, Affinity::After),
            Cursor::new(3, Affinity::Before),
        ))
        .unwrap();
    assert_eq!(selection.rects().len(), 1);
    assert_eq!(
        selection.rects()[0].rect(),
        Rect::new(94.0, 52.0, 12.0, 44.0)
    );
    let Hit::Text(hit) = layout.hit(Point::new(100.0, 85.0)) else {
        panic!("finite centered second X must have an actual physical text hit")
    };
    assert_eq!(hit.source_cursor(), second);
    assert_eq!(
        layout.position_geometry(&hit).unwrap().rect(),
        Rect::new(94.0, 84.0, 12.0, 1.0)
    );
    let query = layout
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    assert_eq!(query.units().len(), 1);
    let unit = query.units().next().unwrap();
    let pieces = unit
        .pieces()
        .map(|piece| {
            let crate::SourceRangeGeometry::Line(piece) = piece else {
                panic!("ordinary Range fixture must retain Line geometry");
            };
            piece
        })
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].rect(), Rect::new(94.0, 84.0, 12.0, 12.0));
    assert_eq!(pieces[0].advance_edges(), (84.0, 96.0));
    assert_eq!(pieces[0].baseline(), 100.0);
    assert_eq!(
        layout.analyzed_source().identity(),
        prepared.analyzed_source().identity()
    );
}
