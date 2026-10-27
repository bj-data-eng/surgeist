//! A real control-only adjacent line cannot be silently skipped by source movement.
use super::*;
use skrifa::MetadataProvider;

#[test]
fn clockwise_source_next_line_refuses_lossy_control_target_after_exact_traversal() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(0x41_u32).unwrap().to_u32(), 35);
    assert_eq!(font.charmap().map(0x42_u32).unwrap().to_u32(), 36);
    let mut s = system_with(bytes.clone());
    let mut value = input(source("AB"), own_style(), false);
    value.controls = (0..2)
        .map(|_| crate::ParagraphControl {
            at: 1,
            kind: crate::ParagraphControlKind::ForcedBreak,
        })
        .collect();
    value.anchor_order = (0..2)
        .map(|control| crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(control),
            affinity: Affinity::After,
        })
        .collect();
    let p = prepare(&mut s, value, CW);
    assert_eq!(p.orientation(), CW);
    assert_eq!(
        p.analyzed_source().source().id(),
        Some(Id::from_u64(1515002))
    );
    assert_eq!(p.analyzed_source().source().revision(), 11);
    assert_eq!(p.analyzed_source().source().text().code_units(), [65, 66]);
    let controls = p
        .control_views()
        .map(|c| {
            assert_eq!(c.kind(), &crate::ParagraphControlKind::ForcedBreak);
            assert_eq!(c.source_position().get(), 1);
            c.handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    assert_ne!(controls[0], controls[1]);
    let forces = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .map(|o| o.handle())
        .collect::<Vec<_>>();
    assert_eq!(forces.len(), 2);
    assert_ne!(forces[0], forces[1]);
    assert_ne!(forces[0].view().boundary(), forces[1].view().boundary());
    for force in &forces {
        assert_eq!(force.view().boundary().view().source_after().get(), 1);
    }
    let first = p
        .select(selected_request(&p, start(&p), forces[0].clone()))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let middle = p
        .select(selected_request(
            &p,
            forces[0].view().boundary(),
            forces[1].clone(),
        ))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let last = p
        .select(selected_request(
            &p,
            forces[1].view().boundary(),
            terminal(&p),
        ))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (
            first.feedback().advance,
            middle.feedback().advance,
            last.feedback().advance
        ),
        (10.0, 0.0, 10.0)
    );
    for (line, ineligibility) in [
        (&first, ProtocolError::NonFormattedForcedLine),
        (&middle, ProtocolError::NonFormattedForcedLine),
        (&last, ProtocolError::NonFormattedInFlowContent),
    ] {
        assert_eq!(line.feedback().line_height, 10.0);
        assert_eq!(
            reason(&line.nonformatted().unwrap_err()),
            &ineligibility,
            "actual Forced middle is formatted, unlike a terminal Paragraph phantom"
        );
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
        }
    }
    assert_eq!(glyphs(&first), [(35, 0.0, 0.0, 0.0, 10.0)]);
    assert!(glyphs(&middle).is_empty());
    assert_eq!(glyphs(&last), [(36, 0.0, 0.0, 0.0, 10.0)]);
    assert_eq!(
        first
            .control_slots()
            .map(|c| c.control())
            .collect::<Vec<_>>(),
        [controls[0].clone()]
    );
    assert_eq!(
        middle
            .control_slots()
            .map(|c| c.control())
            .collect::<Vec<_>>(),
        [controls[1].clone()]
    );
    assert_eq!(last.control_slots().count(), 0);
    // Actual default-font8/2 supplies the before/after extent on each real
    // Forced line; the middle has zero inline advance but retains its receipt.
    let layout = placed(
        &p,
        vec![
            (first, 20.0, 100.0),
            (middle, 20.0, 80.0),
            (last, 20.0, 60.0),
        ],
    );
    assert_eq!(layout.lines().len(), 3);
    assert_eq!(
        layout.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 1), Range::new(1, 2)]
    );
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|l| l.bounds())
            .collect::<Vec<_>>(),
        [
            Rect::new(98.0, 20.0, 10.0, 10.0),
            Rect::new(78.0, 20.0, 10.0, 0.0),
            Rect::new(58.0, 20.0, 10.0, 10.0)
        ]
    );
    assert_eq!(layout.nonformatted_lines().count(), 0);
    let original = Cursor::new(0, Affinity::After);
    assert_eq!(
        layout.cursor(original).unwrap().rect(),
        Rect::new(98.0, 20.0, 10.0, 1.0)
    );
    let exact_start = layout
        .positions()
        .find(|p| p.source_cursor() == original)
        .unwrap();
    assert_eq!(
        layout.position_geometry(&exact_start).unwrap().rect(),
        Rect::new(98.0, 20.0, 10.0, 1.0)
    );
    let exact_middle = layout
        .move_position(&exact_start, Movement::NextLine)
        .unwrap();
    assert_eq!(
        exact_middle.source_cursor(),
        Cursor::new(1, Affinity::After)
    );
    assert_eq!(
        layout.position_geometry(&exact_middle).unwrap().rect(),
        Rect::new(78.0, 20.0, 10.0, 1.0)
    );
    let exact_last = layout
        .move_position(&exact_middle, Movement::NextLine)
        .unwrap();
    assert_eq!(exact_last.source_cursor(), Cursor::new(1, Affinity::After));
    assert_eq!(
        layout.position_geometry(&exact_last).unwrap().rect(),
        Rect::new(58.0, 20.0, 10.0, 1.0)
    );
    assert_ne!(
        exact_middle, exact_last,
        "equal source cursors cannot collapse actual lines"
    );
    let ambiguity = ProtocolError::AmbiguousSourcePosition {
        index: 1,
        affinity: Affinity::After,
    };
    assert_eq!(
        reason(&layout.cursor(exact_middle.source_cursor()).unwrap_err()),
        &ambiguity
    );
    assert_eq!(layout.source().text().code_units(), [65, 66]);
    // The adjacent target exists, but its raw source projection is ambiguous.
    // Ok(original0) silently omits it; Ok(1) invents a unique occurrence.
    let error = layout
        .move_cursor(original, Movement::NextLine)
        .expect_err("raw source NextLine must validate the genuine adjacent anchor target");
    assert_eq!(reason(&error), &ambiguity);
    assert_eq!(
        layout.position_geometry(&exact_middle).unwrap().rect(),
        Rect::new(78.0, 20.0, 10.0, 1.0)
    );
    assert_eq!(layout.source().text().code_units(), [65, 66]);
}
