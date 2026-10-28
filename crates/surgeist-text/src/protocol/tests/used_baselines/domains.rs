//! Real hypothetical/default and selected-indication font operations.
use super::*;

fn system() -> System {
    let bytes = sfnt::replace_tables(
        AHEM,
        vec![(
            *b"BASE",
            base::base(
                0,
                Some(base::Axis {
                    tags: vec![*b"romn"],
                    scripts: vec![
                        base::Script {
                            tag: *b"grek",
                            default_index: 0,
                            coordinates: vec![base::Coord::Design(-100)],
                        },
                        base::Script {
                            tag: *b"latn",
                            default_index: 0,
                            coordinates: vec![base::Coord::Design(100)],
                        },
                    ],
                }),
                None,
                None,
            ),
        )],
    );
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    for (character, glyph) in [('A', 35), (' ', 3), ('0', 18), ('\u{3a5}', 277)] {
        assert_eq!(parsed.charmap().map(character), Some(GlyphId::new(glyph)));
        assert_eq!(
            parsed
                .glyph_metrics(FontSize::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(glyph)),
            Some(1000.0)
        );
    }
    let axis = parsed.base().unwrap().horiz_axis().unwrap().unwrap();
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(
        scripts
            .base_script_records()
            .iter()
            .map(|record| {
                let values = record
                    .base_script(scripts.offset_data())
                    .unwrap()
                    .base_values()
                    .unwrap()
                    .unwrap();
                (
                    record.base_script_tag().to_be_bytes(),
                    values.base_coords().get(0).unwrap().coordinate(),
                )
            })
            .collect::<Vec<_>>(),
        [(*b"grek", -100), (*b"latn", 100)]
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn css_tabs(text: &str, style: Style) -> AnalyzedSource {
    let source = Source::identified(text, Id::from_u64(1561030), 2);
    let len = source.text().len();
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style.clone(),
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: Some(
                    crate::ParagraphTabReference::new(style, ParagraphBaseDirection::LeftToRight)
                        .unwrap(),
                ),
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, len),
                    aggregate: Range::new(0, len),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    )
}

#[test]
fn actual_css_tab_reference_keeps_latin_metrics_and_does_not_merge_distinct_script_origins() {
    let mut system = system();
    for text in ["\t", "\u{3a5}\tA"] {
        let input = css_tabs(
            text,
            Style {
                tab_size: crate::TabSize::spaces(4.0).unwrap(),
                ..own_style()
            },
        );
        for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
            let prepared = system
                .prepare(
                    &input,
                    PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops)
                        .unwrap()
                        .with_orientation(orientation)
                        .with_baseline_reference(ALPHA),
                )
                .unwrap();
            let mut selection = request(&prepared);
            selection.context.orientation = orientation;
            selection.context.tabs = TabContext::CssStops { origin: Some(0.0) };
            selection.context.spacing = SpacingPolicy::None;
            let line = prepared
                .select(selection)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            let only_tab = text == "\t";
            // Four real one-em spaces create a40px interval. Greek10 then
            // tab30 places A at40; the tab/default operation is isolated Latn.
            assert_eq!(line.feedback().advance, if only_tab { 40.0 } else { 50.0 });
            assert_eq!(
                (line.feedback().baseline, line.feedback().line_height),
                if only_tab { (7.0, 10.0) } else { (9.0, 12.0) }
            );
            let layout = placed(
                &prepared,
                line,
                if orientation == LineOrientation::Horizontal {
                    70.0
                } else {
                    100.0
                },
            );
            assert_eq!(
                layout.source().text().code_units(),
                text.encode_utf16().collect::<Vec<_>>()
            );
            let runs = layout.run_views().collect::<Vec<_>>();
            if only_tab {
                assert_eq!(runs.len(), 1);
                assert!(runs[0].glyphs().next().is_none());
                assert_eq!(
                    runs[0].origin(),
                    if orientation == LineOrientation::Horizontal {
                        Point::new(20.0, 71.0)
                    } else {
                        Point::new(99.0, 20.0)
                    }
                );
                let raw = runs[0]
                    .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
                    .unwrap();
                assert_eq!(raw.script().iso15924(), *b"Latn");
            } else {
                assert_eq!(runs.len(), 2);
                assert_eq!(
                    runs.iter().map(|run| run.range()).collect::<Vec<_>>(),
                    [Range::new(0, 1), Range::new(2, 3)]
                );
                assert_eq!(
                    runs.iter().map(|run| run.origin()).collect::<Vec<_>>(),
                    if orientation == LineOrientation::Horizontal {
                        [Point::new(20.0, 69.0), Point::new(60.0, 71.0)]
                    } else {
                        [Point::new(101.0, 20.0), Point::new(99.0, 60.0)]
                    }
                );
                assert_eq!(
                    runs.iter()
                        .flat_map(|run| run
                            .glyphs()
                            .map(|glyph| (glyph.id(), glyph.advance()))
                            .collect::<Vec<_>>())
                        .collect::<Vec<_>>(),
                    [(277, 10.0), (35, 10.0)]
                );
            }
        }
    }
}

fn selected_request(
    prepared: &PreparedText,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
) -> LineSelectionRequest {
    let mut selection = request(prepared);
    selection.context.orientation = prepared.orientation();
    selection.context.spacing = SpacingPolicy::None;
    selection.start = start;
    selection.end = offer.view().boundary();
    let a = selection.start.view().participant_offset();
    let b = selection.end.view().participant_offset();
    selection.participants = prepared.participants().skip(a).take(b - a).collect();
    selection.context.start = if a == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    selection.context.end = if offer.view().kind() == PreparedBreakKind::ParagraphEnd {
        LineEnd::Paragraph
    } else {
        LineEnd::Soft
    };
    selection.termination = crate::PreparedTermination::Ordinary(offer);
    selection
}

#[test]
fn real_selected_greek_indication_uses_its_own_reference_and_retains_original_soft_hyphen_custody()
{
    let mut system = system();
    let source = Source::identified("AA\u{ad}AA", Id::from_u64(1561031), 5);
    let input = analyzed(
        source.clone(),
        Style {
            locale: Some("und-Latn".into()),
            hyphens: crate::Hyphens::Manual,
            hyphenate_limit_chars: crate::HyphenateLimitChars {
                total: crate::HyphenationMinimum::Count(4),
                before: crate::HyphenationMinimum::Count(2),
                after: crate::HyphenationMinimum::Count(2),
            },
            native_hyphenation_convention: Some(
                crate::NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    crate::Text::from("\u{3a5}"),
                    crate::HyphenationPlacement::LogicalBefore,
                )
                .unwrap(),
            ),
            ..own_style()
        },
    );
    let auto_input = analyzed(
        source.clone(),
        Style {
            hyphenate_limit_chars: crate::HyphenateLimitChars::default(),
            ..input.style().clone()
        },
    );
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        // The same actual4/2/2 source cut remains ineligible under native
        // Auto5/2/2. The positive case explicitly captures its smaller minimum.
        let denied = system
            .prepare(
                &auto_input,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(ALPHA),
            )
            .unwrap();
        let denied_cut = denied
            .hyphenation_cut_views()
            .find(|cut| cut.boundary().view().source_before().get() == 3)
            .unwrap();
        assert_eq!(
            denied_cut.counts(),
            Some(crate::HyphenationCounts {
                total: 4,
                before: 2,
                after: 2
            })
        );
        assert_eq!(
            denied_cut.style().hyphenate_limit_chars,
            crate::HyphenateLimitChars::default()
        );
        assert!(denied_cut.opportunity().is_none());
        let prepared = system
            .prepare(
                &input,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(ALPHA),
            )
            .unwrap();
        let cut = prepared
            .hyphenation_cut_views()
            .find(|cut| cut.boundary().view().source_before().get() == 3)
            .unwrap();
        assert_eq!(
            cut.counts(),
            Some(crate::HyphenationCounts {
                total: 4,
                before: 2,
                after: 2
            })
        );
        assert_eq!(
            cut.style().hyphenate_limit_chars,
            crate::HyphenateLimitChars {
                total: crate::HyphenationMinimum::Count(4),
                before: crate::HyphenationMinimum::Count(2),
                after: crate::HyphenationMinimum::Count(2),
            }
        );
        let offer = cut
            .opportunity()
            .expect("actual4/2/2 cut satisfies its explicitly captured4/2/2 minima");
        let alternative = offer.view().alternatives().next().unwrap().handle();
        assert_eq!(alternative.view().text().code_units(), [0x3a5]);
        assert_eq!(alternative.view().source_range(), Some(Range::new(2, 3)));
        let start = prepared.boundary_views().next().unwrap().handle();
        let mut prefix = selected_request(&prepared, start, offer.clone());
        prefix.alternative = Some(alternative.clone());
        let first = prepared
            .select(prefix)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        let terminal = prepared
            .opportunity_views()
            .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let mut suffix = selected_request(&prepared, offer.view().boundary(), terminal);
        suffix.incoming = first.hyphenation_continuation();
        let second = prepared
            .select(suffix)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            first.selected_line().request().alternative,
            Some(alternative)
        );
        assert_eq!(
            (
                first.feedback().advance,
                first.feedback().baseline,
                first.feedback().line_height
            ),
            (30.0, 9.0, 12.0)
        );
        assert_eq!(
            (
                second.feedback().advance,
                second.feedback().baseline,
                second.feedback().line_height
            ),
            (20.0, 7.0, 10.0)
        );
        assert_eq!(
            first
                .run_views()
                .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [35, 35, 277]
        );
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: prepared.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: [
                (
                    first,
                    if orientation == LineOrientation::Horizontal {
                        70.0
                    } else {
                        100.0
                    },
                ),
                (
                    second,
                    if orientation == LineOrientation::Horizontal {
                        90.0
                    } else {
                        80.0
                    },
                ),
            ]
            .into_iter()
            .map(|(line, baseline)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline,
                    },
                    orientation,
                    atomics: vec![],
                })
            })
            .collect(),
        })
        .unwrap();
        assert_eq!(layout.source(), &source);
        let indication = layout
            .run_views()
            .find(|run| run.glyphs().any(|glyph| glyph.id() == 277))
            .unwrap();
        assert_eq!(
            indication.origin(),
            if orientation == LineOrientation::Horizontal {
                Point::new(40.0, 69.0)
            } else {
                Point::new(101.0, 40.0)
            }
        );
        let raw = indication
            .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
            .unwrap();
        assert_eq!(raw.script().iso15924(), *b"Grek");
        assert!(
            matches!(raw.outcome(), FontBaselineOutcome::Available(value) if value.coordinate() == -1.0)
        );
        assert_eq!(
            layout
                .glyph_runs()
                .iter()
                .flat_map(|run| run
                    .glyphs()
                    .iter()
                    .filter(|glyph| glyph.id() == 277)
                    .map(|glyph| glyph.range()))
                .collect::<Vec<_>>(),
            // The alternative retains the authored SHY span above; its drawn
            // indication is inserted at the cut, without claiming authored text.
            [Range::new(3, 3)]
        );
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accessibility_uses_named_line_geometry_and_preserves_original_character_positions() {
    let mut system = system();
    let input = analyzed(Source::new("A"), own_style());
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        let prepared = system
            .prepare(
                &input,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(ALPHA),
            )
            .unwrap();
        let layout = placed(
            &prepared,
            finalize(&prepared),
            if orientation == LineOrientation::Horizontal {
                70.0
            } else {
                100.0
            },
        );
        let mut state = crate::Accessibility::default();
        let parent = accesskit::NodeId(1561032);
        let update = layout
            .accessibility_update(&mut state, parent, parent, Point::default())
            .unwrap();
        let nodes = update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);
        let node = &nodes[0].1;
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            if orientation == LineOrientation::Horizontal {
                (20.0, 63.0, 30.0, 73.0)
            } else {
                (97.0, 20.0, 107.0, 30.0)
            }
        );
        assert_eq!(node.character_widths(), Some(&[10.0][..]));
        assert_eq!(
            node.text_direction(),
            Some(if orientation == LineOrientation::Horizontal {
                accesskit::TextDirection::LeftToRight
            } else {
                accesskit::TextDirection::TopToBottom
            })
        );
        assert_eq!(
            layout
                .access_position(&state, Cursor::new(0, Affinity::After))
                .unwrap()
                .unwrap()
                .character_index,
            0
        );
    }
}
