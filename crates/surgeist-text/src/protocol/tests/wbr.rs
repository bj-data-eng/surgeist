//! Actual control-owned separators: literal metrics, membership and extraction.
use super::*;
use crate::{FinalizedFragmentOwner, Id, LineHeight, Range, WordSpaceTransform};

fn style(operation: WordSpaceTransform, mode: WhiteSpace) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: mode,
        word_space_transform: operation,
        ..Style::default()
    }
}
fn wbr(at: usize, operation: WordSpaceTransform) -> paragraph::ParagraphControl {
    paragraph::ParagraphControl {
        at,
        kind: paragraph::ParagraphControlKind::Wbr {
            style: Box::new(style(operation, WhiteSpace::Preserve)),
        },
    }
}
fn input(
    source: Source,
    mode: WhiteSpace,
    controls: Vec<paragraph::ParagraphControl>,
) -> ParagraphInput {
    let len = source.text().len();
    let mut anchor_order: Vec<_> = (0..controls.len())
        .map(|i| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(i),
            affinity: crate::Affinity::After,
        })
        .collect();
    if len == 0 {
        anchor_order.insert(
            0,
            paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::After,
            },
        );
    }
    ParagraphInput {
        source: source.clone(),
        base_style: style(WordSpaceTransform::None, mode),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls,
        replacements: vec![],
        boundary_policies: vec![],
        anchor_order,
    }
}
fn catalogue() -> System {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(crate::shaping_test_fonts::width_ahem())
        .unwrap();
    s
}
fn prepare(value: ParagraphInput) -> PreparedText {
    let a = AnalyzedSource::from_paragraph(value).unwrap();
    catalogue()
        .prepare(
            &crate::test_wrapping::bind_missing(&a, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn whole(p: &PreparedText) -> FinalizedLine {
    let mut r = request(p);
    r.controls = r
        .participants
        .iter()
        .filter_map(|part| match part {
            PreparedParticipant::Control(c) => Some(ControlAllocation {
                occurrence: c.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), crate::shaping_test_fonts::width_ahem());
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn full_selection(layout: &crate::Layout) -> crate::PlacedSelection {
    let positions: Vec<_> = layout.positions().collect();
    layout
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap()
}

#[test]
fn genuine_wbr_emits_real_control_owned_glyphs_without_source_donation() {
    for (operation, glyph, advance, total) in [
        (WordSpaceTransform::Space, 3, 5.0, 25.0),
        (WordSpaceTransform::IdeographicSpace, 1, 20.0, 40.0),
    ] {
        let source = Source::identified("AA", Id::from_u64(6309), 14);
        let p = prepare(input(
            source.clone(),
            WhiteSpace::Preserve,
            vec![wbr(1, operation)],
        ));
        let control = p.control_views().next().unwrap().handle();
        let control_units: Vec<_> = p
            .unit_views()
            .filter(|u| u.controls().next().is_some())
            .collect();
        assert_eq!(control_units.len(), 1);
        assert_eq!(
            control_units[0].controls().collect::<Vec<_>>().as_slice(),
            std::slice::from_ref(&control)
        );
        assert!(control_units[0].origins().next().is_none());
        assert!(
            !p.participants()
                .any(|part| matches!(part, PreparedParticipant::Control(_)))
        );
        let line = whole(&p);
        assert_eq!(glyphs(&line), [(35, 10.0), (glyph, advance), (35, 10.0)]);
        assert_eq!(line.feedback().advance, total);
        let mapping: Vec<_> = line
            .mapping_views()
            .filter(|m| m.kind() == PresentationMappingKind::GeneratedSeparator)
            .collect();
        assert_eq!(mapping.len(), 1);
        assert_eq!(mapping[0].separator_control(), Some(control.clone()));
        assert_eq!(mapping[0].source_range(), Range::new(1, 1));
        assert!(mapping[0].processed_range().is_empty());
        assert_eq!(
            (
                mapping[0].presentation_range().start(),
                mapping[0].presentation_range().end()
            ),
            (1, 2)
        );
        assert!(mapping[0].alternative().is_none());
        let fragments: Vec<_> = line
            .fragment_views()
            .filter(|f| matches!(f.owner(),FinalizedFragmentOwner::Control(ref c) if c==&control))
            .collect();
        assert_eq!(fragments.len(), 1);
        assert_eq!(
            fragments[0].bounds().unwrap().size.width,
            f64::from(advance)
        );
        assert!(fragments[0].origin().is_none());
        assert_eq!(
            p.analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [0x41, 0x41]
        );
        let layout = place(p, line, 0.0, 8.0).unwrap();
        assert_eq!(layout.source(), &source);
        let selected = full_selection(&layout);
        assert_eq!(selected.source_text().code_units(), [0x41, 0x41]);
        assert_eq!(selected.plaintext().code_units(), [0x41, 0x41]);
    }
}

#[test]
fn equal_anchor_wbrs_keep_distinct_units_and_only_the_last_joins_the_original_mark() {
    let source = Source::identified("A\u{3099}", Id::from_u64(6310), 15);
    let p = prepare(input(
        source.clone(),
        WhiteSpace::Preserve,
        vec![
            wbr(1, WordSpaceTransform::Space),
            wbr(1, WordSpaceTransform::IdeographicSpace),
        ],
    ));
    let origin = p.origin_views().next().unwrap().handle();
    let controls: Vec<_> = p.control_views().map(|c| c.handle()).collect();
    assert_ne!(controls[0], controls[1]);
    let control_units: Vec<_> = p
        .unit_views()
        .filter(|u| u.controls().next().is_some())
        .collect();
    assert_eq!(control_units.len(), 2);
    assert_eq!(
        control_units[0].controls().collect::<Vec<_>>(),
        [controls[0].clone()]
    );
    assert!(control_units[0].origins().next().is_none());
    assert_eq!(
        control_units[1].controls().collect::<Vec<_>>(),
        [controls[1].clone()]
    );
    assert!(control_units[1].origins().next().is_none());
    // Preparation issues scalar contributors. The placed Range observation
    // below, rather than this inventory, groups the actual shared grapheme.
    let mark_units: Vec<_> = p
        .unit_views()
        .filter(|unit| {
            unit.origins()
                .any(|origin| origin.local_range() == Range::new(1, 2))
        })
        .collect();
    assert_eq!(mark_units.len(), 1);
    assert!(mark_units[0].controls().next().is_none());
    assert_eq!(
        mark_units[0]
            .origins()
            .map(|origin| origin.local_range())
            .collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(35, 10.0), (3, 5.0), (1, 20.0), (100, 0.0)]);
    assert_eq!(line.feedback().advance, 35.0);
    let layout = place(p, line, 0.0, 8.0).unwrap();
    let mark = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    assert_eq!(mark.units().len(), 1);
    let unit = mark.units().next().unwrap();
    assert_eq!(unit.controls().collect::<Vec<_>>(), [controls[1].clone()]);
    assert_eq!(
        unit.selected_intersections()
            .map(|i| i.local_range())
            .collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(
        unit.expanded_intersections()
            .map(|i| i.local_range())
            .collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(
        (
            unit.presentation_range().start(),
            unit.presentation_range().end()
        ),
        (2, 4)
    );
    assert_eq!(
        unit.pieces()
            .map(|piece| piece.rect().size.width)
            .sum::<f64>(),
        20.0
    );
    let a = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert!(a.units().all(|u| u.controls().next().is_none()));
    assert_eq!(
        a.units()
            .flat_map(|u| { u.pieces().map(|p| p.rect().size.width).collect::<Vec<_>>() })
            .sum::<f64>(),
        10.0
    );
    assert_eq!(
        full_selection(&layout).source_text().code_units(),
        [0x41, 0x3099]
    );
}

#[test]
fn separator_only_placement_has_five_pixel_geometry_and_empty_original_copy() {
    let source = Source::identified("", Id::from_u64(6311), 16);
    let p = prepare(input(
        source.clone(),
        WhiteSpace::Preserve,
        vec![wbr(0, WordSpaceTransform::Space)],
    ));
    let c = p.control_views().next().unwrap().handle();
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(3, 5.0)]);
    assert_eq!(line.feedback().advance, 5.0);
    assert!(line.fragment_views().any(
        |f| matches!(f.owner(),FinalizedFragmentOwner::Control(ref own) if own==&c)
            && f.bounds().unwrap().size.width == 5.0
    ));
    let layout = place(p, line, 0.0, 8.0).unwrap();
    let positions: Vec<_> = layout.positions().collect();
    let before = positions
        .iter()
        .find(|p| layout.position_geometry(p).unwrap().rect().origin.x == 0.0)
        .unwrap();
    let after = positions
        .iter()
        .find(|p| layout.position_geometry(p).unwrap().rect().origin.x == 5.0)
        .unwrap();
    let selected = layout.position_selection(before, after).unwrap();
    assert!(!selected.is_collapsed());
    assert!(selected.source_text().is_empty());
    assert!(selected.plaintext().is_empty());
    assert!(
        selected
            .source_pieces()
            .iter()
            .all(|p| p.local_range().is_empty())
    );
    assert_eq!(
        selected
            .geometry()
            .rects()
            .iter()
            .map(|r| r.rect().size.width)
            .sum::<f64>(),
        5.0
    );
    assert_eq!(layout.source(), &source);
    #[cfg(feature = "text-accessibility")]
    {
        let mut state = crate::Accessibility::new(6311);
        let parent = 1001_u64.into();
        let update = layout
            .accessibility_update(&mut state, parent, parent, crate::Point::new(0.0, 0.0))
            .unwrap();
        assert!(
            update
                .nodes
                .iter()
                .all(|(_, node)| node.role() != accesskit::Role::TextRun)
        );
        assert!(update.nodes.iter().all(|(_, node)| node.value().is_none()));
    }
}

#[test]
fn deleted_span_wbr_adds_geometry_but_no_original_or_plaintext_characters() {
    let source = Source::identified("A  A", Id::from_u64(6312), 17);
    let p = prepare(input(
        source.clone(),
        WhiteSpace::Discard,
        vec![wbr(2, WordSpaceTransform::Space)],
    ));
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [0x41, 0x41]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(35, 10.0), (3, 5.0), (35, 10.0)]);
    assert_eq!(line.feedback().advance, 25.0);
    let layout = place(p, line, 0.0, 8.0).unwrap();
    assert!(
        layout
            .source_range_fragments(&origin, Range::new(1, 3))
            .unwrap()
            .units()
            .next()
            .is_none()
    );
    let selected = full_selection(&layout);
    assert_eq!(
        selected.source_text().code_units(),
        [0x41, 0x20, 0x20, 0x41]
    );
    assert_eq!(selected.plaintext().code_units(), [0x41, 0x41]);
    assert_eq!(layout.source(), &source);
}

#[test]
fn generated_wbr_space_is_full_width_eligible_in_all_actual_whitespace_modes() {
    for mode in [
        WhiteSpace::Collapse,
        WhiteSpace::Discard,
        WhiteSpace::Preserve,
        WhiteSpace::PreserveSpaces,
    ] {
        let mut value = input(
            Source::new("AA"),
            mode,
            vec![wbr(1, WordSpaceTransform::Space)],
        );
        if let paragraph::ParagraphControlKind::Wbr { style } = &mut value.controls[0].kind {
            style.white_space = mode;
            style.text_transform =
                crate::TextTransform::Set(crate::TextTransformSet::none().with_full_width(true));
        }
        let p = prepare(value);
        let line = whole(&p);
        assert_eq!(
            glyphs(&line),
            [(35, 10.0), (1, 20.0), (35, 10.0)],
            "{mode:?}"
        );
        assert_eq!(line.feedback().advance, 40.0);
    }
}

#[test]
fn unexpanded_wbr_keeps_its_own_wrap_and_is_not_an_optional_break_conversion() {
    for wrap in [crate::Wrap::Word, crate::Wrap::None] {
        let mut value = input(
            Source::new("AA"),
            WhiteSpace::Preserve,
            vec![wbr(1, WordSpaceTransform::None)],
        );
        value.base_style.wrap = if wrap == crate::Wrap::Word {
            crate::Wrap::None
        } else {
            crate::Wrap::Word
        };
        if let paragraph::ParagraphControlKind::Wbr { style } = &mut value.controls[0].kind {
            style.wrap = wrap;
        }
        let p = prepare(value);
        assert!(matches!(
            p.control_views().next().unwrap().kind(),
            paragraph::ParagraphControlKind::Wbr { .. }
        ));
        assert_eq!(
            p.participants()
                .filter(|p| matches!(p, PreparedParticipant::Control(_)))
                .count(),
            1
        );
        assert_eq!(
            p.opportunity_views()
                .filter(|o| o.kind() == PreparedBreakKind::Allowed)
                .count(),
            usize::from(wrap == crate::Wrap::Word)
        );
        let line = whole(&p);
        assert_eq!(glyphs(&line), [(35, 10.0), (35, 10.0)]);
        assert_eq!(line.feedback().advance, 20.0);
        assert!(
            line.mapping_views()
                .all(|m| m.kind() != PresentationMappingKind::GeneratedSeparator)
        );
    }
    let mut value = input(
        Source::new("AA"),
        WhiteSpace::Preserve,
        vec![paragraph::ParagraphControl {
            at: 1,
            kind: paragraph::ParagraphControlKind::OptionalBreak {
                wrap: crate::Wrap::Word,
            },
        }],
    );
    value.base_style.word_space_transform = WordSpaceTransform::Space;
    assert_eq!(whole(&prepare(value)).feedback().advance, 20.0);
}

#[test]
#[cfg(feature = "text-accessibility")]
fn wbr_accessibility_keeps_authored_values_and_real_source_glyph_positions() {
    let source = Source::identified("AA", Id::from_u64(6313), 18);
    for operation in [WordSpaceTransform::None, WordSpaceTransform::Space] {
        let p = prepare(input(
            source.clone(),
            WhiteSpace::Preserve,
            vec![wbr(1, operation)],
        ));
        let line = whole(&p);
        assert_eq!(
            line.feedback().advance,
            if operation == WordSpaceTransform::None {
                20.0
            } else {
                25.0
            }
        );
        let layout = place(p, line, 0.0, 8.0).unwrap();
        let mut state = crate::Accessibility::new(6309);
        let parent = 1001_u64.into();
        let update = layout
            .accessibility_update(&mut state, parent, parent, crate::Point::new(0.0, 0.0))
            .unwrap();
        let values: Vec<_> = update.nodes.iter().filter_map(|(_, n)| n.value()).collect();
        assert_eq!(values.concat(), "AA");
        assert!(!values.iter().any(|v| v.contains(' ')));
        assert_eq!(layout.source(), &source);
    }
}
