//! Public UTF-16 source and interaction contracts over registered fixture fonts.
use surgeist_text::*;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn registered_ahem() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .expect("the registered-only catalogue initializes");
    system
        .register_fonts(AHEM.to_vec())
        .expect("the retained Ahem fixture registers");
    system
}

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 20.0,
        ..Style::default()
    }
}

fn options() -> Options {
    Options {
        quantize: false,
        ..Options::default()
    }
}

fn layout(system: &mut System, text: impl Into<Text>) -> Layout {
    system
        .layout(Source::new(text), style(), options())
        .expect("scalar source reaches the real registered-font layout path")
}

#[test]
fn supplementary_and_multibyte_bmp_ranges_count_source_units() {
    let mut system = registered_ahem();
    // Ahem has UPEM 1000 and 1000-unit advances for A, X, É and glyph 0.
    // The supplementary scalar is absent from its cmap and uses that glyph 0;
    // no system fallback participates. Neither UTF-8 bytes nor scalar counts
    // describe the supplementary source range 1..3.
    let supplementary = layout(
        &mut system,
        Text::from_code_units(vec![0x0041, 0xd83d, 0xde00, 0x0058]),
    );
    let bmp = layout(&mut system, "AÉX");
    for (output, expected_ranges, expected_ids, units) in [
        (
            &supplementary,
            [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)],
            [35, 0, 58],
            4,
        ),
        (
            &bmp,
            [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)],
            [35, 100, 58],
            3,
        ),
    ] {
        let glyphs: Vec<_> = output
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs().iter().copied())
            .collect();
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.range()).collect::<Vec<_>>(),
            expected_ranges
        );
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.id()).collect::<Vec<_>>(),
            expected_ids
        );
        assert_eq!(
            glyphs
                .iter()
                .map(|glyph| glyph.advance())
                .collect::<Vec<_>>(),
            [20.0, 20.0, 20.0]
        );
        assert_eq!(output.lines().len(), 1);
        assert_eq!(output.lines()[0].range(), Range::new(0, units));
        assert_eq!(output.source().text().len(), units);
        assert_eq!(output.metrics().full_width(), 60.0);
        for run in output.glyph_runs() {
            assert_eq!(run.font().bytes(), AHEM);
        }
    }
}

#[test]
fn pair_interior_keeps_exact_endpoint_and_projects_geometry_by_affinity() {
    let mut system = registered_ahem();
    let output = layout(&mut system, "A😀X");
    let source = output.source().text();
    assert_eq!(SourcePosition::try_new(source, 2).unwrap().get(), 2);
    assert_eq!(
        SourceRange::try_new(source, 2, 2).unwrap().range(),
        Range::new(2, 2)
    );
    for (affinity, expected_x) in [(Affinity::Before, 20.0), (Affinity::After, 40.0)] {
        let requested = Cursor::new(2, affinity);
        let geometry = output.cursor(requested).unwrap();
        assert_eq!(requested.index(), 2);
        assert_eq!(requested.affinity(), affinity);
        assert_eq!(geometry.rect().origin.x, expected_x);
        let collapsed = Selection::collapsed(requested);
        assert_eq!(collapsed.range(), Range::new(2, 2));
        assert!(output.selection(collapsed).unwrap().rects().is_empty());
    }
    let whole = output
        .selection(Selection::new(
            Cursor::new(1, Affinity::After),
            Cursor::new(3, Affinity::Before),
        ))
        .unwrap();
    assert_eq!(whole.rects().len(), 1);
    assert_eq!(whole.rects()[0].rect().origin.x, 20.0);
    assert_eq!(whole.rects()[0].rect().size.width, 20.0);
    for (start, end) in [(1, 2), (2, 3)] {
        let requested = Selection::new(
            Cursor::new(start, Affinity::After),
            Cursor::new(end, Affinity::Before),
        );
        assert_eq!(output.selection(requested).unwrap(), whole);
        assert_eq!(requested.range(), Range::new(start, end));
    }
}

#[test]
fn editing_can_split_and_rejoin_a_pair_without_repairing_retained_source() {
    let original = Source::new(Text::from_code_units(vec![0x0041, 0xd83d, 0xde00, 0x0058]));
    let split = TextEdit::delete(&original, Range::new(1, 2))
        .unwrap()
        .apply_to(original.clone())
        .unwrap();
    assert_eq!(split.text().code_units(), [0x0041, 0xde00, 0x0058]);
    assert_eq!(split.revision(), 1);
    assert_eq!(
        SourceRange::try_new(split.text(), 1, 2).unwrap().range(),
        Range::new(1, 2)
    );
    let insert = TextEdit::insert(&split, 1, Text::from_code_units(vec![0xd83d])).unwrap();
    assert_eq!(insert.inserted_text().code_units(), [0xd83d]);
    let rejoined = insert.apply_to(split.clone()).unwrap();
    assert_eq!(rejoined.text(), original.text());
    assert_eq!(rejoined.revision(), 2);
    assert_eq!(
        original.text().code_units(),
        [0x0041, 0xd83d, 0xde00, 0x0058]
    );
    assert_eq!(original.revision(), 0);
    assert_eq!(split.text().code_units(), [0x0041, 0xde00, 0x0058]);
    let mut system = registered_ahem();
    assert!(system.layout(rejoined, style(), options()).is_ok());
}

#[test]
fn actual_refusal_preserves_original_cache_while_isolated_and_literal_units_remain_distinct() {
    let mut system = registered_ahem();
    let old = layout(&mut system, "AX");
    let warm = layout(&mut system, "AX");
    assert_eq!(warm.key(), old.key());
    let before = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    let refused = Source::new(format!("B\u{0315}{}", "\u{0300}".repeat(32)));
    let error = system
        .layout(refused.clone(), style(), options())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::ShapingRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::ShapingRefused {
            reason: surgeist_text::ShapingRefusalReason::ContextLimit,
        })
    );
    assert_eq!(system.stats(), before);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    assert_eq!(refused.text().len(), 34);
    assert_eq!(old.source().text().code_units(), [0x0041, 0x0058]);
    assert_eq!(layout(&mut system, "AX").key(), old.key());
    assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
    let isolated = Source::new(Text::from_code_units(vec![0x0041, 0xd800, 0x0058]));
    let output = system.layout(isolated.clone(), style(), options()).unwrap();
    assert_eq!(output.source(), &isolated);
    assert_eq!(isolated.text().code_units(), [0x0041, 0xd800, 0x0058]);
    let literal = layout(&mut system, "A�X");
    assert_eq!(
        literal.source().text().code_units(),
        [0x0041, 0xfffd, 0x0058]
    );
    assert_ne!(output.source().text(), literal.source().text());
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accessibility_reuses_actual_child_focus_and_keeps_host_bytes_distinct_from_source_units() {
    use accesskit::{NodeId, Role};

    let mut system = registered_ahem();
    let output = layout(&mut system, "AÉ😀X");
    assert_eq!(output.source().text().len(), 5);
    let parent = NodeId(1000);
    let mut state = Accessibility::new(10);
    let first = output
        .accessibility_update(&mut state, parent, parent, Point::default())
        .unwrap();
    let children = first
        .nodes
        .iter()
        .find(|(id, _)| *id == parent)
        .unwrap()
        .1
        .children()
        .to_vec();
    assert!(!children.is_empty());
    let mut host_text = String::new();
    let mut host_lengths = Vec::new();
    for child in &children {
        let node = &first.nodes.iter().find(|(id, _)| id == child).unwrap().1;
        assert_eq!(node.role(), Role::TextRun);
        let value = node.value().unwrap();
        assert_eq!(
            node.character_lengths()
                .iter()
                .map(|length| usize::from(*length))
                .sum::<usize>(),
            value.len()
        );
        host_text.push_str(value);
        host_lengths.extend_from_slice(node.character_lengths());
    }
    assert_eq!(host_text, "AÉ😀X");
    assert_eq!(host_lengths, [1, 2, 4, 1]);
    let child_focus = children[0];
    let second = output
        .accessibility_update(&mut state, parent, child_focus, Point::default())
        .unwrap();
    assert_eq!(second.focus, child_focus);
    assert_eq!(
        second
            .nodes
            .iter()
            .find(|(id, _)| *id == parent)
            .unwrap()
            .1
            .children(),
        children
    );
    let requested = Cursor::new(3, Affinity::Before);
    let retained_position = output.access_position(&state, requested).unwrap().unwrap();
    assert_eq!(
        Some(retained_position),
        output
            .access_position(&state, Cursor::new(2, Affinity::Before))
            .unwrap()
    );
    let error = output
        .accessibility_update(&mut state, child_focus, child_focus, Point::default())
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::AccessibilityIdentityConflict { id: child_focus.0 })
    );
    assert_eq!(
        output.access_position(&state, requested).unwrap(),
        Some(retained_position)
    );
    let recovered = output
        .accessibility_update(&mut state, parent, child_focus, Point::default())
        .unwrap();
    assert_eq!(recovered.focus, child_focus);
    assert_eq!(
        recovered
            .nodes
            .iter()
            .find(|(id, _)| *id == parent)
            .unwrap()
            .1
            .children(),
        children
    );
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accessibility_omits_character_geometry_for_actual_styled_grapheme_pieces() {
    use accesskit::{NodeId, TextPosition};

    let mut system = registered_ahem();
    // The base and combining acute form one grapheme despite the scalar-aligned
    // size boundary. This is a source-unit projection test, not a complex-font
    // shaping claim: the retained Ahem fixture supplies the only font.
    let mut source = Source::new("a\u{301}X");
    source.span(
        Range::new(1, 2),
        Style {
            size: 30.0,
            ..style()
        },
    );
    let output = system.layout(source, style(), options()).unwrap();
    assert_eq!(
        output.source().text().code_units(),
        [0x0061, 0x0301, 0x0058]
    );

    let runs = output.glyph_runs();
    let pieces: Vec<_> = runs
        .iter()
        .filter(|run| run.range().start < 2 && !run.glyphs().is_empty())
        .collect();
    assert_eq!(pieces.len(), 2, "the fixture must shape two real pieces");
    assert_eq!(pieces[0].range(), Range::new(0, 1));
    assert_eq!(pieces[1].range(), Range::new(1, 2));
    assert_eq!(pieces[0].font_size(), 20.0);
    assert_eq!(pieces[1].font_size(), 30.0);
    for piece in &pieces {
        assert_eq!(piece.font().bytes(), AHEM);
    }
    let clusters = output.clusters();
    let pieces: Vec<_> = clusters
        .iter()
        .filter(|cluster| cluster.range().start < 2)
        .collect();
    assert_eq!(pieces.len(), 2, "two positioned clusters must reach AX");
    assert_eq!(pieces[0].range(), Range::new(0, 1));
    assert_eq!(pieces[1].range(), Range::new(1, 2));
    assert!(pieces.iter().all(|piece| {
        let bounds = piece.bounds();
        bounds.origin.x.is_finite()
            && bounds.origin.y.is_finite()
            && bounds.size.width.is_finite()
            && bounds.size.height > 0.0
    }));

    let mut state = Accessibility::new(10);
    let parent = NodeId(1000);
    let update = output
        .accessibility_update(&mut state, parent, parent, Point::default())
        .unwrap();
    let (unit_id, unit) = update
        .nodes
        .iter()
        .find(|(_, node)| node.value() == Some("a\u{301}"))
        .expect("both styled pieces retain one original grapheme");
    assert_eq!(unit.character_lengths(), [3]);
    assert_eq!(unit.character_positions(), None);
    assert_eq!(unit.character_widths(), None);
    let (single_id, single) = update
        .nodes
        .iter()
        .find(|(_, node)| node.value() == Some("X"))
        .expect("the adjacent scalar remains a separate unit");
    assert_eq!(single.character_lengths(), [1]);
    assert_eq!(single.character_positions(), Some([0.0_f32].as_slice()));
    // Ahem X advances 1000 units at UPEM 1000, hence 20 at size 20.
    assert_eq!(single.character_widths(), Some([20.0_f32].as_slice()));

    let start = TextPosition {
        node: *unit_id,
        character_index: 0,
    };
    let end = TextPosition {
        node: *unit_id,
        character_index: 1,
    };
    let following_start = TextPosition {
        node: *single_id,
        character_index: 0,
    };
    let upstream = Cursor::new(1, Affinity::Before);
    let downstream = Cursor::new(1, Affinity::After);
    assert_eq!(upstream.index(), 1);
    assert_eq!(downstream.index(), 1);
    assert_eq!(
        output.access_position(&state, upstream).unwrap(),
        Some(start)
    );
    let downstream_position = output.access_position(&state, downstream).unwrap().unwrap();
    // At source boundary 2, the preceding unit's end and following unit's
    // start are equivalent host encodings. Neither changes the source endpoint.
    assert!(downstream_position == end || downstream_position == following_start);
    assert_eq!(
        output.cursor(upstream).unwrap(),
        output.cursor(Cursor::new(0, Affinity::Before)).unwrap()
    );
    assert_eq!(
        output.cursor(downstream).unwrap(),
        output.cursor(Cursor::new(2, Affinity::After)).unwrap()
    );
    let selection = |start, end| {
        Selection::new(
            Cursor::new(start, Affinity::After),
            Cursor::new(end, Affinity::Before),
        )
    };
    let whole = output.selection(selection(0, 2)).unwrap();
    assert!(!whole.rects().is_empty());
    for (from, to) in [(0, 2), (0, 1), (1, 2)] {
        let requested = selection(from, to);
        assert_eq!(requested.range(), Range::new(from, to));
        assert_eq!(output.selection(requested).unwrap(), whole);
        let projected = output.access_selection(&state, requested).unwrap().unwrap();
        assert_eq!(projected.anchor, start);
        assert_eq!(projected.focus, end);
    }
}

#[path = "utf16_interactions/original_surrogate_analysis.rs"]
mod original_surrogate_analysis;

#[path = "utf16_interactions/font_admission.rs"]
mod font_admission;

#[path = "utf16_interactions/forced_break_adjacency.rs"]
mod forced_break_adjacency;

#[path = "utf16_interactions/unicode_hard_separators.rs"]
mod unicode_hard_separators;
