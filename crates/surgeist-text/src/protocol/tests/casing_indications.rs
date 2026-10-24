//! Selected indications retain their own operation and genuine source authority.
use super::*;

fn upper(automatic: bool) -> Style {
    let locale = if automatic { "en-US" } else { "und-Latn" };
    Style {
        text_transform: crate::TextTransform::Set(
            crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
        ),
        hyphens: if automatic {
            Hyphens::Auto
        } else {
            Hyphens::Manual
        },
        locale: Some(locale.into()),
        native_hyphenation_convention: Some(
            NativeHyphenationConvention::language_script(
                locale,
                "Latn",
                Text::from("a"),
                HyphenationPlacement::BothSides,
            )
            .unwrap(),
        ),
        ..own_style("a", HyphenationPlacement::BothSides)
    }
}

fn position(
    layout: &crate::Layout,
    x: f64,
    y: f64,
    affinity: crate::Affinity,
) -> crate::PlacedPosition {
    layout
        .positions()
        .filter(|position| {
            let geometry = layout.position_geometry(position).unwrap().rect();
            geometry.origin.x == x
                && geometry.origin.y == y
                && position.source_cursor().affinity() == affinity
        })
        .last()
        .expect("actual selected indication edge")
}

#[test]
fn transformed_source_and_automatic_both_sides_do_not_recase_indications_or_fabricate_coverage() {
    for automatic in [false, true] {
        let source = Source::new(if automatic {
            "associate"
        } else {
            "ab\u{ad}cde"
        });
        let p = capture(source.clone(), upper(automatic), AHEM);
        let at = if automatic { 4 } else { 3 };
        let offer = cut(
            &p,
            at,
            if automatic {
                counts(4, 5)
            } else {
                counts(2, 3)
            },
        );
        let first = finish(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
        )
        .unwrap();
        let second = finish(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                end(&p),
                first.hyphenation_continuation(),
                false,
            ),
        )
        .unwrap();
        assert_eq!(
            first.inner.selected.presentation.projection.text,
            if automatic { "ASSOa" } else { "ABa" }
        );
        assert_eq!(
            second.inner.selected.presentation.projection.text,
            if automatic { "aCIATE" } else { "aCDE" }
        );
        assert_eq!(
            (first.feedback().advance, second.feedback().advance),
            if automatic {
                (50.0, 60.0)
            } else {
                (30.0, 40.0)
            }
        );
        let layout =
            place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
        let positions: Vec<_> = layout.positions().collect();
        let whole = layout
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap();
        assert_eq!(
            whole.source_text(),
            source.text(),
            "Source SHY is extracted once across its two emissions"
        );
        for (x, y) in [(if automatic { 40.0 } else { 20.0 }, 0.0), (0.0, 10.0)] {
            let selected = layout
                .position_selection(
                    &position(&layout, x, y, crate::Affinity::After),
                    &position(&layout, x + 10.0, y, crate::Affinity::Before),
                )
                .unwrap();
            assert_eq!(
                selected.source_text().code_units(),
                if automatic { &[][..] } else { &[0xad][..] }
            );
            assert_eq!(selected.geometry().rects()[0].rect().size.width, 10.0);
        }
    }
}

#[test]
fn transformed_control_both_sides_keeps_emissions_source_less() {
    let source = Source::new("abcde");
    let p = capture_input(
        ParagraphInput {
            source: source.clone(),
            base_style: upper(false),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 5),
                aggregate: Range::new(0, 5),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            controls: vec![paragraph::ParagraphControl {
                at: 2,
                kind: paragraph::ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(upper(false)),
                },
            }],
            anchor_order: vec![paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Control(0),
                affinity: crate::Affinity::After,
            }],
        },
        AHEM,
    );
    let offer = cut(&p, 2, counts(2, 3));
    assert!(matches!(
        alternative(&offer).view().definition(),
        crate::PreparedAlternativeDefinition::Control { .. }
    ));
    let first = finish(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
    )
    .unwrap();
    let second = finish(
        &p,
        selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        ),
    )
    .unwrap();
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (30.0, 40.0)
    );
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
    let positions: Vec<_> = layout.positions().collect();
    assert_eq!(
        layout
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text(),
        source.text()
    );
    let selected = layout
        .position_selection(
            &position(&layout, 0.0, 10.0, crate::Affinity::After),
            &position(&layout, 10.0, 10.0, crate::Affinity::Before),
        )
        .unwrap();
    assert!(selected.source_text().is_empty());
    assert_eq!(selected.geometry().rects()[0].rect().size.width, 10.0);
}

#[test]
fn real_barrier_before_expansion_does_not_split_its_internal_ss_ligature_on_indication_selection() {
    // Real optional GSUB SS(53,53)->58; no dependency on missing-glyph geometry.
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[53, 53],
        &[
            (0x41, 35),
            (0x42, 36),
            (0x43, 37),
            (0x44, 38),
            (0x45, 39),
            (0x53, 53),
            (0x61, 67),
            (0x62, 68),
            (0x63, 69),
            (0x64, 70),
            (0x65, 71),
            (0xdf, 134),
        ],
    );
    let glyphs = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let literal = capture(
        Source::new("SS"),
        own_style("a", HyphenationPlacement::BothSides),
        &bytes,
    );
    let control = finish(
        &literal,
        selected_request(&literal, start(&literal), end(&literal), None, false),
    )
    .unwrap();
    assert_eq!(glyphs(&control), [58]);
    assert_eq!(control.feedback().advance, 10.0);

    let source = Source::new("bßA\u{ad}cde");
    let p = capture_input(
        ParagraphInput {
            source: source.clone(),
            base_style: upper(false),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![
                paragraph::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(0, 1),
                    aggregate: Range::new(0, 1),
                    kind: paragraph::ParagraphOriginKind::Authored,
                },
                paragraph::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(1, 7),
                    aggregate: Range::new(1, 7),
                    kind: paragraph::ParagraphOriginKind::Authored,
                },
            ],
            connections: vec![paragraph::ParagraphConnection {
                at: 1,
                kind: paragraph::ParagraphConnectionKind::Disconnected,
            }],
            boundary_policies: vec![],
            controls: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        &bytes,
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        &[0x62, 0xdf, 0x41, 0xad, 0x63, 0x64, 0x65]
    );
    let unbroken = finish(&p, selected_request(&p, start(&p), end(&p), None, false)).unwrap();
    assert_eq!(glyphs(&unbroken), [36, 58, 35, 37, 38, 39]);
    assert_eq!(unbroken.feedback().advance, 60.0);
    let offer = p
        .hyphenation_cut_views()
        .find(|view| view.boundary().view().source_before().get() == 4)
        .expect("real original source SHY cut")
        .opportunity()
        .expect("actual word counts admit explicit cut");
    let chosen = alternative(&offer);
    let crate::PreparedAlternativeDefinition::Source { origin, source } =
        chosen.view().definition()
    else {
        panic!("actual source SHY definition");
    };
    assert_eq!(source, Range::new(3, 4));
    assert_eq!(origin.view().aggregate_range(), Range::new(1, 7));
    let line = finish(&p, selected_request(&p, start(&p), offer, None, true)).unwrap();
    assert_eq!(
        line.selected_line().request().alternative.as_ref(),
        Some(&chosen)
    );
    // ß contributes two S emissions, but a barrier exists only BEFORE that
    // component. Choosing the real indication must keep its 10px SS ligature.
    assert_eq!(
        (glyphs(&line), line.feedback().advance),
        (vec![36, 58, 35, 67], 40.0)
    );
}
