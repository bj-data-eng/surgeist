//! Ordinary ligature closure and mixed ordinary/Cell AX endpoints.

#![cfg(feature = "text-accessibility")]

use super::*;
use crate::protocol::tests::combined_cells::{
    inner_tabs::provider_ranges::qualified_optional_liga, widths::tracking_resources,
};

fn ordinary_style() -> Style {
    let font = Font::new().family("Ahem").feature("\"liga\" 1");
    assert_eq!(font.features, ["\"liga\" 1"]);
    Style {
        font,
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        wrap: Wrap::Word,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        text_transform: TextTransform::none(),
        text_combine_upright: TextCombineUpright::None,
        ..Style::default()
    }
}

#[test]
fn ordinary_authored_optional_liga_keeps_host_egcs_but_placed_ax_preserves_shared_glyph_closure() {
    // Maintained real liga font: fi35/36->58/1000 at UPEM1000, with public
    // Complete+Optional provenance and a liga0 unligated control. Added native
    // metric tables preserve that actual cmap/GSUB/outline/advance resource.
    let bytes = qualified_optional_liga();
    let source = Source::identified("fi", Id::from_u64(7_841_611_001), 1);
    let analyzed = AnalyzedSource::new(
        source.clone(),
        ordinary_style(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.combined_cells().len(), 0);
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(analyzed.inner.host_projection.graphemes, [0, 1, 2]);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let layout = system
        .layout_analyzed(
            &analyzed,
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap();
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    assert_eq!(layout.analyzed_source().combined_cells().len(), 0);
    let actual = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, [(58, 10.0)]);
    assert_eq!(
        layout
            .inner
            .lines
            .iter()
            .flat_map(|line| {
                line.finalized
                    .data
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.glyph.source)
            })
            .collect::<Vec<_>>(),
        [Range::new(0, 2)],
    );
    let parent = NodeId(1);
    let mut state = Accessibility::new(90_000);
    let source_update = analyzed
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    let source_units = original_units(&source_update, parent);
    assert_eq!(
        source_units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["f", "i"]
    );
    // Host EGC1 remains logical source1 in the font-independent phase.
    assert_eq!(
        analyzed
            .access_position(&state, Cursor::new(1, Affinity::Before))
            .unwrap(),
        Some(source_units[0].2)
    );
    assert_eq!(
        analyzed
            .access_position(&state, Cursor::new(1, Affinity::After))
            .unwrap(),
        Some(source_units[1].1)
    );
    let partial = Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(1, Affinity::Before),
    );
    assert_eq!(
        analyzed.access_selection(&state, partial).unwrap(),
        Some(TextSelection {
            anchor: source_units[0].1,
            focus: source_units[0].2,
        })
    );

    let placed = layout
        .accessibility_update(&mut state, parent, parent, Point::new(0.0, 0.0))
        .unwrap();
    let units = original_units(&placed, parent);
    assert_eq!(
        units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["f", "i"]
    );
    // Ordinary behavior is its actual shared-glyph/interaction owner: interval
    //0..2 removes interior1 from placed legal stops. Before snaps0, After2.
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(1, Affinity::Before))
            .unwrap(),
        Some(units[0].1)
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(1, Affinity::After))
            .unwrap(),
        Some(units[1].2)
    );
    assert_eq!(
        layout.access_selection(&state, partial).unwrap(),
        Some(TextSelection {
            anchor: units[0].1,
            focus: units[1].2,
        })
    );
    assert_eq!(
        layout
            .access_selection(&state, Selection::new(partial.focus(), partial.anchor()))
            .unwrap(),
        Some(TextSelection {
            anchor: units[1].2,
            focus: units[0].1,
        })
    );
    for (affinity, expected) in [
        (Affinity::Before, units[0].1),
        (Affinity::After, units[1].2),
    ] {
        let cursor = Cursor::new(1, affinity);
        assert_eq!(
            layout
                .access_selection(&state, Selection::new(cursor, cursor))
                .unwrap(),
            Some(TextSelection {
                anchor: expected,
                focus: expected
            })
        );
    }
}

#[test]
fn mixed_ordinary_shared_glyph_and_cell_keep_each_endpoint_under_its_actual_ax_owner() {
    let parent_bytes = qualified_optional_liga();
    let body_bytes = composition_font();
    let outside = ordinary_style();
    let inside = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        text_combine_upright: TextCombineUpright::All,
        ..outside.clone()
    };
    let mixed = tracking_resources::prepare_mixed(
        "fiAA",
        Range::new(2, 4),
        7_841_611_002,
        parent_bytes.clone(),
        body_bytes.clone(),
        outside,
        inside,
    )
    .unwrap();
    let prepared = &mixed.prepared;
    let analyzed = prepared.analyzed_source();
    assert_eq!(analyzed.source().text(), &Text::from("fiAA"));
    assert_eq!(analyzed.inner.host_projection.graphemes, [0, 1, 2, 3, 4]);
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x41]);
    assert_eq!(
        cell.source_pieces()
            .map(|piece| piece.aggregate())
            .collect::<Vec<_>>(),
        [Range::new(2, 3), Range::new(3, 4)]
    );
    let line = complete_line(prepared);
    assert_eq!(line.feedback().advance, 20.0); // ordinary fi10 + fixed Cell10, tracking0.
    let mut ordinary = Vec::new();
    let mut inner = Vec::new();
    for run in line.run_views() {
        if run.cell().is_some() {
            assert_eq!(run.face().bytes(), body_bytes.as_slice());
            inner.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        } else {
            assert_eq!(run.face().bytes(), parent_bytes.as_slice());
            ordinary.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        }
    }
    assert_eq!(ordinary, [(Range::new(0, 2), 58, 10.0)]);
    assert_eq!(
        inner,
        [(Range::new(2, 3), 36, 5.0), (Range::new(3, 4), 36, 5.0)]
    );
    let layout = placed_vertical_cell(prepared, line);
    let parent = NodeId(1);
    let mut state = Accessibility::new(100_000);
    let source = analyzed
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    let source_units = original_units(&source, parent);
    assert_eq!(
        source_units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["f", "i", "A", "A"]
    );
    let selection = Selection::new(
        Cursor::new(1, Affinity::After),
        Cursor::new(3, Affinity::Before),
    );
    // Existing-passing SOURCE-only control: exact i + first Cell A,1..3.
    assert_eq!(
        analyzed.access_selection(&state, selection).unwrap(),
        Some(TextSelection {
            anchor: source_units[1].1,
            focus: source_units[2].2,
        })
    );
    // Placed endpoints retain the actual ordinary and Cell conversion owners.
    let placed = layout
        .accessibility_update(&mut state, parent, parent, Point::new(0.0, 0.0))
        .expect("placed mixed AX must retain ordinary closure and exact logical Cell endpoints");
    let units = original_units(&placed, parent);
    assert_eq!(
        units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        ["f", "i", "A", "A"]
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(1, Affinity::Before))
            .unwrap(),
        Some(units[0].1)
    );
    // Ordinary snapping reaches cut2; preserved After affinity chooses the
    // following Cell A's start, while Before at that cut keeps the i's end.
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(1, Affinity::After))
            .unwrap(),
        Some(units[2].1)
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(2, Affinity::Before))
            .unwrap(),
        Some(units[1].2)
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(2, Affinity::After))
            .unwrap(),
        Some(units[2].1)
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(3, Affinity::Before))
            .unwrap(),
        Some(units[2].2)
    );
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(3, Affinity::After))
            .unwrap(),
        Some(units[3].1)
    );
    // Only the ordinary endpoint expands:1->0. The Cell endpoint remains3,
    // not its physical far edge4. Authored source coverage is therefore0..3.
    assert_eq!(
        layout.access_selection(&state, selection).unwrap(),
        Some(TextSelection {
            anchor: units[0].1,
            focus: units[2].2,
        })
    );
    assert_eq!(
        layout
            .access_selection(
                &state,
                Selection::new(selection.focus(), selection.anchor())
            )
            .unwrap(),
        Some(TextSelection {
            anchor: units[2].2,
            focus: units[0].1,
        })
    );
}
