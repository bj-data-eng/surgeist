use super::*;

fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .expect("finite registered catalogue");
    system
        .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .expect("actual Ahem fixture");
    system
}

fn layout(system: &mut System, text: impl Into<Text>) -> Layout {
    system
        .layout(
            Source::new(text),
            Style {
                font: Font::new().family("Ahem"),
                size: 20.0,
                ..Style::default()
            },
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .expect("scalar source uses the real provider")
}

fn ahem_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 20.0,
        ..Style::default()
    }
}

#[test]
fn forced_box_only_line_uses_its_real_anchor_in_nonempty_source() {
    let mut system = system();
    let mut source = Source::new("A");
    source.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::InFlow,
        0,
        Size::new(20.0, 20.0),
    ));
    let layout = system
        .layout(
            source,
            ahem_style(),
            Options {
                width: Some(20.0),
                quantize: false,
                ..Options::default()
            },
        )
        .expect("a box followed by real Ahem text remains supported");
    let lines = layout.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].range(), Range::new(0, 0));
    assert_eq!(lines[1].range(), Range::new(0, 1));
    let runs = layout.glyph_runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].line(), 1);
    assert_eq!(runs[0].glyphs().len(), 1);
    assert_eq!(runs[0].glyphs()[0].range(), Range::new(0, 1));
    assert_eq!(layout.source().text().code_units(), [u16::from(b'A')]);
}

#[test]
fn exact_units_survive_copy_and_edits_that_split_and_join_pairs() {
    let text = Text::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xfffd]);
    let retained = text.clone();
    let original = Source::identified(text, Id::from_u64(8), 3);
    let edited = TextEdit::replace(
        &original,
        Range::new(1, 2),
        Text::from_code_units(vec![0xdc00]),
    )
    .expect("unit range")
    .apply_to(original.clone())
    .expect("exact edit");
    assert_eq!(edited.text().code_units(), [0xd800, 0xdc00, 0xdc00, 0xfffd]);
    assert_eq!(retained.code_units(), [0xd800, 0x61, 0xdc00, 0xfffd]);
    assert_eq!(original.text(), &retained);
    assert_eq!(edited.revision(), 4);
    let pair = Source::new("😀");
    let half = TextEdit::delete(&pair, Range::new(1, 2))
        .expect("half-pair range")
        .apply_to(pair.clone())
        .expect("lossless deletion");
    assert_eq!(half.text().code_units(), [0xd83d]);
    let joined = TextEdit::insert(&half, 1, Text::from_code_units(vec![0xde00]))
        .expect("exact insertion")
        .apply_to(half)
        .expect("real adjacency");
    assert_eq!(joined.text(), pair.text());
}

#[test]
fn composer_marks_count_units_and_can_establish_real_pair_adjacency() {
    let mut composer = compose();
    let high = composer.push(Text::from_code_units(vec![0xd83d]));
    let low = composer.push(Text::from_code_units(vec![0xde00]));
    assert_eq!(high.range(), Range::new(0, 1));
    assert_eq!(low.range(), Range::new(1, 2));
    let source = composer.finish();
    assert_eq!(source.text().code_units(), [0xd83d, 0xde00]);
    let mut system = system();
    assert_eq!(
        layout(&mut system, source.text().clone()).lines()[0].range(),
        Range::new(0, 2)
    );
}

#[test]
fn empty_source_retains_real_cursor_metrics_without_publishing_a_metrics_character() {
    let mut system = system();
    let layout = layout(&mut system, "");
    assert!(layout.source().text().is_empty());
    assert!(
        layout
            .lines()
            .iter()
            .all(|line| line.range() == Range::new(0, 0))
    );
    assert!(layout.clusters().is_empty());
    assert!(
        layout
            .glyph_runs()
            .iter()
            .all(|run| run.glyphs().is_empty() && run.range() == Range::new(0, 0))
    );
    assert!(
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .expect("empty real font metrics")
            .rect()
            .size()
            .height
            > 0.0
    );
    assert!(
        layout
            .selection(Selection::collapsed(Cursor::new(0, Affinity::After)))
            .expect("empty selection")
            .rects()
            .is_empty()
    );
    #[cfg(feature = "text-accessibility")]
    {
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1000);
        let update = layout
            .accessibility_update(&mut state, parent, parent, Point::default())
            .expect("empty host projection");
        for (_, node) in update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
        {
            assert_eq!(node.value(), Some(""));
            assert!(node.character_lengths().is_empty());
        }
        let selection = layout
            .access_selection(
                &state,
                Selection::collapsed(Cursor::new(0, Affinity::After)),
            )
            .expect("empty mapping")
            .expect("real empty-line position");
        assert_eq!(selection.anchor, selection.focus);
        assert_eq!(selection.focus.character_index, 0);
    }
}

#[test]
fn supplementary_and_multibyte_provider_ranges_return_source_units() {
    let mut system = system();
    let layout = layout(&mut system, "é😀x");
    assert_eq!(layout.source().text().len(), 4);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 4));
    let ranges: Vec<_> = layout
        .clusters()
        .into_iter()
        .map(|cluster| cluster.range())
        .collect();
    for expected in [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)] {
        assert!(ranges.contains(&expected), "{ranges:?}");
    }
    assert!(
        layout
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs())
            .all(|glyph| glyph.range().end <= 4)
    );
}

#[test]
fn genuine_provider_context_refusal_retains_original_source_and_warm_publication() {
    let mut system = system();
    let old = layout(&mut system, "AX");
    let old_lines = old.lines();
    let stats = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    let source = Source::identified(
        format!("B\u{0315}{}", "\u{0300}".repeat(32)),
        Id::from_u64(17),
        4,
    );
    let units = source.text().code_units().to_vec();
    let error = system
        .layout(source.clone(), ahem_style(), Options::default())
        .expect_err("33-mark required reorder is a real provider context refusal");
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::ShapingRefused {
            reason: ShapingRefusalReason::ContextLimit,
        })
    );
    assert_eq!(source.text().code_units(), units);
    assert_eq!(source.id(), Some(Id::from_u64(17)));
    assert_eq!(source.revision(), 4);
    assert_eq!(system.stats(), stats);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    assert_eq!(old.source().text().code_units(), [0x41, 0x58]);
    assert_eq!(old.lines(), old_lines);
    assert_eq!(layout(&mut system, "AX").key(), old.key());
}

#[test]
fn pair_interior_style_and_box_boundaries_are_precise_atomic_shaping_refusals() {
    let mut system = system();
    let stats = system.stats();
    for (range, kind) in [
        (Range::new(1, 2), ShapingBoundary::SpanStart),
        (Range::new(0, 1), ShapingBoundary::SpanEnd),
    ] {
        let mut source = Source::new("😀");
        source.span(range, Style::default());
        let error = system
            .layout(source, Style::default(), Options::default())
            .expect_err("cannot split shaping scalar");
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::UnsupportedShapingBoundary { index: 1, kind })
        );
    }
    let mut source = Source::new("😀");
    source.inline_box(InlineBox::new(
        Id::from_u64(2),
        InlineBoxKind::InFlow,
        1,
        Size::new(4.0, 4.0),
    ));
    let error = system
        .layout(source, Style::default(), Options::default())
        .expect_err("cannot insert inside scalar");
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedShapingBoundary {
            index: 1,
            kind: ShapingBoundary::InlineBox
        })
    );
    assert_eq!(system.stats(), stats);
}

fn assert_partial_geometry(layout: &Layout, full: Range, partial: Range) {
    let selection = |range: Range| {
        Selection::new(
            Cursor::new(range.start, Affinity::After),
            Cursor::new(range.end, Affinity::Before),
        )
    };
    assert!(
        !layout
            .selection(selection(full))
            .expect("actual full geometry")
            .rects()
            .is_empty()
    );
    assert_eq!(
        layout
            .selection(selection(partial))
            .expect("partial source selection"),
        layout
            .selection(selection(full))
            .expect("complete typographic unit")
    );
}

#[test]
fn pair_interior_affinities_project_both_sides_without_changing_the_request() {
    let mut system = system();
    let layout = layout(&mut system, "😀X");
    let before = Cursor::new(1, Affinity::Before);
    let after = Cursor::new(1, Affinity::After);
    assert_eq!(before.index(), 1);
    assert_eq!(after.index(), 1);
    assert_eq!(
        layout.cursor(before).expect("interior before"),
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .expect("preceding stop")
    );
    assert_eq!(
        layout.cursor(after).expect("interior after"),
        layout
            .cursor(Cursor::new(2, Affinity::After))
            .expect("following stop")
    );
    assert_partial_geometry(&layout, Range::new(0, 2), Range::new(0, 1));
    assert_partial_geometry(&layout, Range::new(0, 2), Range::new(1, 2));
    assert!(
        layout
            .selection(Selection::collapsed(after))
            .expect("collapsed interior")
            .rects()
            .is_empty()
    );
    assert!(
        layout
            .selection(Selection::new(before, after))
            .expect("equal logical endpoints remain collapsed")
            .rects()
            .is_empty()
    );
}

#[test]
fn combining_grapheme_partial_selection_covers_all_actual_style_pieces() {
    let mut system = system();
    let mut source = Source::new("a\u{301}X");
    source.span(
        Range::new(1, 2),
        Style {
            size: 30.0,
            ..ahem_style()
        },
    );
    let layout = system
        .layout(source, ahem_style(), Options::default())
        .expect("scalar style boundary remains supported");
    assert_partial_geometry(&layout, Range::new(0, 2), Range::new(0, 1));
    assert_partial_geometry(&layout, Range::new(0, 2), Range::new(1, 2));
    let next = layout
        .move_cursor(Cursor::new(0, Affinity::After), Movement::NextCluster)
        .expect("actual legal movement");
    assert_ne!(next.index(), 1, "movement must not expose a split grapheme");
}

#[test]
fn zwj_grapheme_and_crlf_keep_exact_endpoints_with_complete_unit_geometry() {
    let mut system = system();
    let layout = layout(&mut system, "👩\u{200d}👩\r\nX");
    assert_eq!(layout.source().text().len(), 8);
    assert_partial_geometry(&layout, Range::new(0, 5), Range::new(2, 3));
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::Before))
            .expect("interior source stop"),
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .expect("preceding legal stop")
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::After))
            .expect("interior source stop"),
        layout
            .cursor(Cursor::new(5, Affinity::After))
            .expect("following legal stop")
    );
    assert!(
        layout
            .selection(Selection::collapsed(Cursor::new(2, Affinity::After)))
            .expect("collapsed exact endpoint")
            .rects()
            .is_empty()
    );
    let selection = |range: Range| {
        Selection::new(
            Cursor::new(range.start, Affinity::After),
            Cursor::new(range.end, Affinity::Before),
        )
    };
    assert_eq!(
        layout
            .selection(selection(Range::new(5, 6)))
            .expect("half CRLF"),
        layout
            .selection(selection(Range::new(5, 7)))
            .expect("complete CRLF unit")
    );
    #[cfg(feature = "text-accessibility")]
    {
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1000);
        let update = layout
            .accessibility_update(&mut state, parent, parent, Point::default())
            .expect("host text follows same source units");
        let node = &update
            .nodes
            .iter()
            .find(|(_, node)| node.value() == Some("👩\u{200d}👩"))
            .expect("one legal source grapheme")
            .1;
        assert_eq!(node.character_lengths(), [11]);
    }
}

#[test]
fn rtl_pair_carets_project_logically_before_visual_geometry() {
    let mut system = system();
    let layout = layout(&mut system, "א😀ב");
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::Before))
            .expect("interior"),
        layout
            .cursor(Cursor::new(1, Affinity::Before))
            .expect("preceding logical stop")
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::After))
            .expect("interior"),
        layout
            .cursor(Cursor::new(3, Affinity::After))
            .expect("following logical stop")
    );
    assert_partial_geometry(&layout, Range::new(1, 3), Range::new(2, 3));
}

#[test]
fn invalid_cursor_requests_refuse_and_old_projection_survives_edit_and_refresh() {
    let mut system = system();
    let layout = layout(&mut system, "é😀X");
    let lines = layout.lines();
    assert_eq!(
        layout
            .cursor(Cursor::new(5, Affinity::After))
            .expect_err("out of units")
            .code,
        ErrorCode::InvalidRange
    );
    let edited = layout
        .apply(Edit::Delete {
            range: Range::new(1, 2),
        })
        .expect("half-pair edit is exact");
    assert_eq!(edited.text().code_units(), [0xe9, 0xde00, 0x58]);
    system.refresh_fonts().expect("real catalogue refresh");
    drop(system);
    assert_eq!(layout.lines(), lines);
    assert_eq!(
        layout.source().text().code_units(),
        [0xe9, 0xd83d, 0xde00, 0x58]
    );
    assert!(layout.cursor(Cursor::new(2, Affinity::After)).is_ok());
}

#[cfg(feature = "text-accessibility")]
mod accessibility {
    use super::*;

    #[test]
    fn mixed_locale_omits_only_language_and_retains_actual_uniform_font_metadata() {
        let mut system = system();
        let mut source = Source::new("a\u{301}");
        source.span(
            Range::new(1, 2),
            Style {
                locale: Some("fr".into()),
                ..ahem_style()
            },
        );
        let layout = system
            .layout(
                source,
                Style {
                    locale: Some("en".into()),
                    ..ahem_style()
                },
                Options::default(),
            )
            .expect("actual same-font locale pieces");
        assert!(!layout.clusters().is_empty());
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1000);
        let update = layout
            .accessibility_update(&mut state, parent, parent, Point::default())
            .expect("one legal selectable grapheme");
        let node = &update
            .nodes
            .iter()
            .find(|(_, node)| node.value() == Some("a\u{301}"))
            .expect("complete original unit")
            .1;
        assert_eq!(node.language(), None);
        assert_eq!(node.font_family(), Some("Ahem"));
        assert_eq!(node.font_size(), Some(20.0));
        assert_eq!(node.font_weight(), Some(400.0));
    }

    #[test]
    fn real_child_focus_is_reused_and_foreign_layout_mapping_is_rejected() {
        let mut system = system();
        let first = layout(&mut system, "é😀X");
        let mut state = Accessibility::new(1);
        let parent = accesskit::NodeId(1);
        let update = first
            .accessibility_update(&mut state, parent, parent, Point::default())
            .expect("owned host projection");
        let child = update
            .nodes
            .iter()
            .find(|(_, node)| node.role() == accesskit::Role::TextRun)
            .expect("actual child")
            .0;
        assert_ne!(child, parent);
        let again = first
            .accessibility_update(&mut state, parent, child, Point::default())
            .expect("retained actual child focus is valid");
        assert_eq!(again.focus, child);
        assert_eq!(
            update.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            again.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
        );
        let pair = first
            .access_selection(
                &state,
                Selection::new(
                    Cursor::new(1, Affinity::After),
                    Cursor::new(2, Affinity::Before),
                ),
            )
            .expect("half pair host join")
            .expect("actual positions");
        assert_eq!(pair.anchor.node, pair.focus.node);
        assert_eq!(pair.anchor.character_index, 0);
        assert_eq!(pair.focus.character_index, 1);
        let collapsed = first
            .access_selection(
                &state,
                Selection::collapsed(Cursor::new(2, Affinity::After)),
            )
            .expect("collapsed")
            .expect("positions");
        assert_eq!(collapsed.anchor, collapsed.focus);
        let other = layout(&mut system, "different");
        assert_eq!(
            other
                .access_position(&state, Cursor::new(0, Affinity::After))
                .expect_err("foreign layout")
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
        assert!(
            first
                .clone()
                .access_position(&state, Cursor::new(0, Affinity::After))
                .expect("same layout clone")
                .is_some()
        );
    }

    #[test]
    fn host_text_lengths_are_bytes_with_legal_combining_crlf_and_style_boundaries() {
        let mut system = system();
        let mut source = Source::new("a\u{301}\r\n😀X");
        source.span(
            Range::new(1, 2),
            Style {
                size: 30.0,
                locale: Some("fr".into()),
                ..ahem_style()
            },
        );
        let layout = system
            .layout(
                source,
                Style {
                    locale: Some("en".into()),
                    ..ahem_style()
                },
                Options::default(),
            )
            .expect("mixed actual style pieces");
        assert!(
            !layout.clusters().is_empty(),
            "actual registered font must shape the fixture"
        );
        let mut state = Accessibility::default();
        let update = layout
            .accessibility_update(
                &mut state,
                accesskit::NodeId(1000),
                accesskit::NodeId(1000),
                Point::default(),
            )
            .expect("scalar AX remains supported");
        let values: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
            .map(|(_, node)| {
                let text = node.value().expect("host scalar text");
                assert_eq!(
                    node.character_lengths()
                        .iter()
                        .map(|len| usize::from(*len))
                        .sum::<usize>(),
                    text.len()
                );
                text.to_owned()
            })
            .collect();
        let mixed = update
            .nodes
            .iter()
            .find(|(_, node)| node.value() == Some("a\u{301}"))
            .expect("actual mixed-locale grapheme unit");
        assert_eq!(
            mixed.1.language(),
            None,
            "mixed source locales cannot be attributed to the whole unit"
        );
        assert!(
            values.contains(&"a\u{301}".to_owned()),
            "actual host units: {values:?}; clusters: {:?}",
            layout.clusters()
        );
        assert!(values.contains(&"\r\n".to_owned()));
        assert_eq!(values.concat(), "a\u{301}\r\n😀X");
    }

    #[test]
    fn absent_selected_font_does_not_erase_original_selectable_graphemes() {
        let analyzed = AnalyzedSource::new(Source::new("ab"), Style::default())
            .expect("original source analysis needs no font");
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1000);
        let update = analyzed
            .accessibility_update(&mut state, parent, parent)
            .expect("source host units do not require invented glyphs");
        let lengths: Vec<_> = update
            .nodes
            .iter()
            .flat_map(|(_, node)| node.character_lengths().iter().copied())
            .collect();
        assert_eq!(lengths, [1, 1]);
        assert_eq!(analyzed.source().text().code_units(), [0x61, 0x62]);
        assert!(update.nodes.iter().all(|(_, node)| node.bounds().is_none()
            && node.font_family().is_none()
            && node.font_size().is_none()
            && node.character_positions().is_none()
            && node.character_widths().is_none()
            && node.next_on_line().is_none()
            && node.previous_on_line().is_none()));
        let position = analyzed
            .access_position(&state, Cursor::new(1, Affinity::After))
            .expect("original legal stop")
            .expect("actual host position");
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == position.node)
            .expect("mapped node")
            .1;
        assert_eq!(node.value(), Some("b"));
        assert_eq!(position.character_index, 0);
        let selection = analyzed
            .access_selection(
                &state,
                Selection::new(
                    Cursor::new(1, Affinity::After),
                    Cursor::new(2, Affinity::Before),
                ),
            )
            .unwrap()
            .unwrap();
        assert_eq!(selection.anchor, position);
        assert_eq!(selection.focus.node, position.node);
        assert_eq!(selection.focus.character_index, 1);
    }

    #[test]
    fn empty_analyzed_source_has_a_real_empty_host_value_without_geometry() {
        let analyzed = AnalyzedSource::new(Source::new(""), ahem_style()).unwrap();
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1000);
        let update = analyzed
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let values: Vec<_> = update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect();
        assert_eq!(values, [""]);
        assert!(update.nodes.iter().all(|(_, node)| node.bounds().is_none()));
        let cursor = analyzed
            .access_position(&state, Cursor::new(0, Affinity::After))
            .unwrap()
            .unwrap();
        assert_eq!(cursor.character_index, 0);
        let selection = analyzed
            .access_selection(
                &state,
                Selection::collapsed(Cursor::new(0, Affinity::Before)),
            )
            .unwrap()
            .unwrap();
        assert_eq!(selection.anchor, selection.focus);
        assert_eq!(selection.anchor, cursor);
    }

    #[test]
    fn analyzed_and_placed_accessibility_share_source_but_keep_exact_phase_bindings() {
        let analyzed =
            AnalyzedSource::new(Source::identified("AX", Id::from_u64(31), 7), ahem_style())
                .unwrap();
        let same = analyzed.clone();
        let foreign = AnalyzedSource::new(analyzed.source().clone(), ahem_style()).unwrap();
        let mut system = system();
        let mut state = Accessibility::new(10);
        let parent = accesskit::NodeId(1000);
        let logical = analyzed
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let logical_ids: Vec<_> = logical.nodes.iter().map(|(id, _)| *id).collect();
        let point = analyzed
            .access_position(&state, Cursor::new(1, Affinity::After))
            .unwrap();
        assert_eq!(
            same.access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap(),
            point
        );
        assert_eq!(
            foreign
                .access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
        let placed = system
            .layout_analyzed(
                &analyzed,
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &placed.inner.prepared.analyzed,
            &analyzed.inner
        ));
        let geometric = placed
            .accessibility_update(&mut state, parent, parent, Point::new(0.25, 0.5))
            .unwrap();
        assert_eq!(
            geometric
                .nodes
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            logical_ids
        );
        assert!(
            geometric
                .nodes
                .iter()
                .all(|(_, node)| node.bounds().is_some())
        );
        assert_eq!(
            analyzed
                .access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
        assert!(
            placed
                .access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap()
                .is_some()
        );
        let other_placement = system
            .layout_analyzed(
                &analyzed,
                Options {
                    width: Some(100.0),
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            other_placement
                .access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
        let cleared = same
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        assert_eq!(
            cleared.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            logical_ids
        );
        assert!(cleared.nodes.iter().all(|(_, node)| node.bounds().is_none()
            && node.font_family().is_none()
            && node.font_size().is_none()
            && node.character_positions().is_none()
            && node.character_widths().is_none()
            && node.next_on_line().is_none()
            && node.previous_on_line().is_none()));
        assert_eq!(
            placed
                .access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
        assert_eq!(
            analyzed
                .access_position(&state, Cursor::new(1, Affinity::After))
                .unwrap(),
            point
        );
        drop(system);
        assert_eq!(analyzed.source().text().code_units(), [0x41, 0x58]);
        assert_eq!(placed.source().revision(), 7);
    }

    #[test]
    fn original_ill_formed_host_refusal_preserves_prior_phase_binding_and_ids() {
        let original = AnalyzedSource::new(Source::new("A"), ahem_style()).unwrap();
        let invalid = AnalyzedSource::new(
            Source::new(Text::from_code_units(vec![0x41, 0xd800])),
            ahem_style(),
        )
        .unwrap();
        let mut state = Accessibility::new(10);
        let parent = accesskit::NodeId(1000);
        let old = original
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let before = original
            .access_position(&state, Cursor::new(0, Affinity::After))
            .unwrap();
        assert_eq!(
            invalid
                .accessibility_update(&mut state, parent, parent)
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityIllFormedSource {
                index: 1,
                unit: 0xd800
            })
        );
        let mut system = system();
        let placed = system
            .layout_analyzed(&invalid, Options::default())
            .unwrap();
        assert_eq!(
            placed
                .accessibility_update(&mut state, parent, parent, Point::default())
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityIllFormedSource {
                index: 1,
                unit: 0xd800
            })
        );
        assert_eq!(
            original
                .access_position(&state, Cursor::new(0, Affinity::After))
                .unwrap(),
            before
        );
        let repeated = original
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        assert_eq!(
            old.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            repeated.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
        );
        assert_eq!(invalid.source().text().code_units(), [0x41, 0xd800]);
    }

    #[test]
    fn indivisible_255_byte_host_unit_succeeds_and_256_refuses_atomically() {
        let allowed = AnalyzedSource::new(
            Source::new(format!("a{}", "\u{301}".repeat(127))),
            ahem_style(),
        )
        .unwrap();
        let refused = AnalyzedSource::new(
            Source::new(format!("é{}", "\u{301}".repeat(127))),
            ahem_style(),
        )
        .unwrap();
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1000);
        let update = allowed
            .accessibility_update(&mut state, parent, parent)
            .expect("255 bytes fit actual u8 host field");
        assert!(
            update
                .nodes
                .iter()
                .any(|(_, node)| node.character_lengths() == [255])
        );
        let previous = allowed
            .access_position(&state, Cursor::new(0, Affinity::After))
            .expect("existing map");
        let error = refused
            .accessibility_update(&mut state, parent, parent)
            .expect_err("one unit cannot fit host length");
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::UnsupportedAccessibilityUnit {
                range: Range::new(0, 128),
                bytes: 256
            })
        );
        assert_eq!(
            allowed
                .access_position(&state, Cursor::new(0, Affinity::After))
                .expect("refusal retains map"),
            previous
        );
        let repeated = allowed
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        assert_eq!(
            update.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            repeated.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn node_id_last_issue_then_refusal_preserves_actual_old_focus_and_mapping() {
        let mut system = system();
        let first = layout(&mut system, "A");
        let second = layout(&mut system, "B");
        let mut state = Accessibility::new(u64::MAX);
        let parent = accesskit::NodeId(1);
        let update = first
            .accessibility_update(&mut state, parent, parent, Point::default())
            .expect("last real child identity");
        let child = update
            .nodes
            .iter()
            .find(|(_, node)| node.role() == accesskit::Role::TextRun)
            .expect("child")
            .0;
        assert_eq!(child, accesskit::NodeId(u64::MAX));
        assert_eq!(
            first
                .accessibility_update(&mut state, parent, child, Point::default())
                .expect("reuse requires no issuance")
                .focus,
            child
        );
        assert_eq!(
            second
                .accessibility_update(&mut state, parent, parent, Point::default())
                .expect_err("identity exhausted")
                .detail(),
            Some(&ErrorDetail::AccessibilityIdentityExhausted)
        );
        assert_eq!(
            first
                .access_position(&state, Cursor::new(0, Affinity::After))
                .expect("actual old mapping")
                .expect("position")
                .node,
            child
        );
        assert_eq!(
            first
                .accessibility_update(&mut state, child, child, Point::default())
                .expect_err("parent cannot alias child")
                .detail(),
            Some(&ErrorDetail::AccessibilityIdentityConflict { id: u64::MAX })
        );
    }
}
