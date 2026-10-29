//! Authored spacing qualification beyond the published three-case checkpoint.
use super::*;
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, Script},
    script::ScriptWithExtensions,
};

fn style(word_spacing: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        word_spacing,
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn input(source: &Source, own: Style, cuts: &[usize]) -> ParagraphInput {
    ParagraphInput {
        source: source.clone(),
        base_style: own,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: cuts
            .windows(2)
            .map(|pair| crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(pair[0], pair[1]),
                aggregate: Range::new(pair[0], pair[1]),
                kind: crate::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: cuts[1..cuts.len() - 1]
            .iter()
            .map(|&at| crate::ParagraphConnection {
                at,
                kind: crate::ParagraphConnectionKind::Continuous,
            })
            .collect(),
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn analyze(value: ParagraphInput, orientation: LineOrientation) -> AnalyzedSource {
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            value,
            if orientation == LineOrientation::Vertical {
                crate::AnalysisTypography::Upright {
                    computed_direction: ParagraphBaseDirection::RightToLeft,
                }
            } else {
                crate::AnalysisTypography::HorizontalComposition
            },
        )
        .unwrap(),
        crate::Wrap::Word,
    )
}

fn prepared(
    system: &mut System,
    source: &Source,
    own: Style,
    orientation: LineOrientation,
    cuts: &[usize],
) -> PreparedText {
    let analysis = if cuts.len() > 2 {
        // The actual common Block supplies the wrapping facts for these
        // connected storage occurrences. Their Connection event is physical
        // chronology inside the EGC, not a shaping or character boundary.
        use crate::SourceProcessingEvent as E;
        let wrap = own.wrap;
        let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
        for origin in 0..cuts.len() - 1 {
            if origin != 0 {
                events.push(E::Connection(origin - 1));
            }
            events.push(E::Origin {
                origin,
                affinity: Affinity::After,
            });
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let processed = std::sync::Arc::new(
            crate::process_contextual_source(crate::ContextualSourceInput {
                source: source.clone(),
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                envelope: crate::SourceProcessingEnvelope::Complete,
                origins: cuts
                    .windows(2)
                    .map(|pair| crate::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: Range::new(pair[0], pair[1]),
                        aggregate: Range::new(pair[0], pair[1]),
                        kind: crate::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: (0..cuts.len() - 2)
                    .map(|before_origin| crate::SourceProcessingConnection {
                        before_origin,
                        after_origin: before_origin + 1,
                        kind: crate::ParagraphConnectionKind::Continuous,
                    })
                    .collect(),
                controls: vec![],
                boundary_policies: (0..cuts.len() - 2)
                    .map(|connection| crate::ParagraphBoundaryPolicy {
                        definition: crate::ParagraphBoundaryDefinition::Connection(connection),
                        preceding_wrap: wrap,
                        following_wrap: wrap,
                        ancestor_wrap: wrap,
                    })
                    .collect(),
                replacements: vec![],
                computed_line_boundaries: vec![],
                scopes: vec![crate::SourceProcessingScope::Block {
                    topology: crate::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: own,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                }],
                paragraphs: vec![crate::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        crate::test_wrapping::bind_missing(
            &AnalyzedSource::from_processed_paragraph(
                processed.paragraphs().next().unwrap(),
                if orientation == LineOrientation::Vertical {
                    crate::AnalysisTypography::Upright {
                        computed_direction: ParagraphBaseDirection::RightToLeft,
                    }
                } else {
                    crate::AnalysisTypography::HorizontalComposition
                },
            )
            .unwrap(),
            wrap,
        )
    } else {
        analyze(input(source, own, cuts), orientation)
    };
    let prepared = system
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    if cuts.len() > 2 {
        let origins = prepared.origin_views().collect::<Vec<_>>();
        assert_eq!(origins.len(), cuts.len() - 1);
        for (origin, pair) in origins.iter().zip(cuts.windows(2)) {
            assert!(origin.capture_position().is_some());
            assert_eq!(origin.source(), source);
            assert_eq!(origin.local_range(), Range::new(pair[0], pair[1]));
            assert_eq!(origin.aggregate_range(), Range::new(pair[0], pair[1]));
        }
        assert_ne!(origins[0].handle(), origins[1].handle());
        assert_eq!(prepared.analyzed_source().source(), source);
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
        );
    }
    prepared
}

fn register(bytes: Vec<u8>) -> System {
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    for tag in [*b"glyf", *b"loca", *b"hhea"] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(AHEM, tag));
    }
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn place_at(
    prepared: &PreparedText,
    line: &FinalizedLine,
    inline: f64,
) -> crate::Result<crate::Layout> {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: inline,
                baseline: if prepared.orientation() == LineOrientation::Horizontal {
                    70.0
                } else {
                    100.0
                },
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    })
}

#[test]
fn all_seven_separator_scalars_keep_complete_utf16_units_and_fixed_width_spaces_remain_unspaced() {
    for separator in [
        ' ',
        '\u{a0}',
        '\u{1361}',
        '\u{10100}',
        '\u{10101}',
        '\u{1039f}',
        '\u{1091f}',
    ] {
        let bytes = mapped(separator, false);
        let mut system = super::system(&bytes, separator, false, false);
        let source = Source::identified(format!("A{separator}A"), Id::from_u64(1564010), 1);
        let len = source.text().len();
        let end = 1 + separator.len_utf16();
        let ranges = [Range::new(0, 1), Range::new(1, end), Range::new(end, len)];
        for orientation in FRAMES {
            let control = prepared(&mut system, &source, style(0.0), orientation, &[0, len]);
            assert_eq!(
                control.analyzed_source().graphemes().collect::<Vec<_>>(),
                ranges
            );
            let natural = finalize(&control);
            assert_eq!(natural.feedback().advance, 30.0);
            assert_eq!(
                raw(&natural)
                    .iter()
                    .map(|row| (row.0, row.4))
                    .collect::<Vec<_>>(),
                ranges.map(|range| (range, 10.0))
            );
            let p = prepared(&mut system, &source, style(4.0), orientation, &[0, len]);
            let line = finalize(&p);
            assert_eq!(raw(&line), raw(&natural));
            assert_eq!(line.feedback().advance, 34.0);
            assert_eq!(paints(&line), [0.0, 12.0, 24.0]);
            let layout = place_at(&p, &line, 20.0).unwrap();
            assert_eq!(layout.source(), &source);
            let selected = layout
                .selection(Selection::new(
                    Cursor::new(1, Affinity::After),
                    Cursor::new(end, Affinity::Before),
                ))
                .unwrap();
            assert_eq!(selected.rects().len(), 1);
            assert_eq!(
                selected.rects()[0].rect(),
                match orientation {
                    LineOrientation::Horizontal => Rect::new(30.0, 62.0, 14.0, 10.0),
                    LineOrientation::Sideways => Rect::new(98.0, 30.0, 10.0, 14.0),
                    LineOrientation::Vertical => Rect::new(95.0, 30.0, 10.0, 14.0),
                }
            );
            assert_eq!(
                layout
                    .glyph_runs()
                    .iter()
                    .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
                    .collect::<Vec<_>>(),
                ranges
            );
        }
    }
    let excluded = (0x2000..=0x200a)
        .chain([0x3000, 0x2c, 0x2e])
        .collect::<Vec<_>>();
    let mut mappings = vec![(0x41, 35)];
    mappings.extend(excluded.iter().map(|&cp| (cp, 58)));
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[82, 82], &mappings);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    for &cp in &excluded {
        assert_eq!(parsed.charmap().map(cp), Some(GlyphId::new(58)));
    }
    let mut system = register(bytes);
    for cp in excluded {
        let source = Source::new(format!("A{}A", char::from_u32(cp).unwrap()));
        for orientation in FRAMES {
            let p = prepared(&mut system, &source, style(4.0), orientation, &[0, 3]);
            let line = finalize(&p);
            assert_eq!(line.feedback().advance, 30.0, "U+{cp:04X} {orientation:?}");
            assert_eq!(paints(&line), [0.0, 10.0, 20.0]);
        }
    }
}

fn marked_font(separator: char, mark: char) -> Vec<u8> {
    let mappings = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[
            (0x41, 35),
            (u32::from(separator), 35),
            (u32::from(mark), 100),
        ],
    );
    let bytes = sfnt::replace_tables(
        &crate::shaping_test_fonts::mark_ahem(),
        vec![(*b"cmap", sfnt::table(&mappings, *b"cmap").to_vec())],
    );
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.charmap().map(separator), Some(GlyphId::new(35)));
    assert_eq!(parsed.charmap().map(mark), Some(GlyphId::new(100)));
    assert!(parsed.gpos().is_ok());
    assert!(parsed.gdef().is_ok());
    bytes
}

#[test]
fn appendix_e_marked_units_keep_real_attachment_and_do_not_split_at_storage_origins() {
    let categories = CodePointMapData::<GeneralCategory>::new();
    assert_eq!(categories.get(' '), GeneralCategory::SpaceSeparator);
    assert_eq!(categories.get('\u{a0}'), GeneralCategory::SpaceSeparator);
    assert_eq!(
        categories.get('\u{1361}'),
        GeneralCategory::OtherPunctuation
    );
    assert_eq!(categories.get('\u{301}'), GeneralCategory::NonspacingMark);
    assert_eq!(categories.get('\u{20dd}'), GeneralCategory::EnclosingMark);
    assert!(!ScriptWithExtensions::new().has_script('\u{20dd}', Script::Common));
    for (separator, mark, spaced) in [
        (' ', '\u{301}', false),
        ('\u{a0}', '\u{301}', false),
        ('\u{1361}', '\u{301}', true),
        ('\u{1361}', '\u{20dd}', true),
    ] {
        let bytes = marked_font(separator, mark);
        let mut system = register(bytes);
        let source = Source::identified(format!("A{separator}{mark}A"), Id::from_u64(1564011), 2);
        for cuts in [&[0, 4][..], &[0, 2, 4][..]] {
            // Only storage origins split the actual base+mark EGC; the
            // Continuous connection and identical styles keep joint shaping.
            for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
                let control = prepared(&mut system, &source, style(0.0), orientation, cuts);
                assert_eq!(
                    control.analyzed_source().graphemes().collect::<Vec<_>>(),
                    [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
                );
                let natural = finalize(&control);
                assert_eq!(
                    raw(&natural),
                    [
                        (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                        (Range::new(1, 3), 35, 0.0, 0.0, 10.0),
                        (Range::new(1, 3), 100, -7.0, -4.0, 0.0),
                        (Range::new(3, 4), 35, 0.0, 0.0, 10.0)
                    ]
                );
                let p = prepared(&mut system, &source, style(4.0), orientation, cuts);
                let line = finalize(&p);
                assert_eq!(raw(&line), raw(&natural));
                assert_eq!(line.feedback().advance, if spaced { 34.0 } else { 30.0 });
                assert_eq!(
                    paints(&line),
                    if spaced {
                        [0.0, 12.0, 22.0, 24.0]
                    } else {
                        [0.0, 10.0, 20.0, 20.0]
                    }
                );
                let layout = place_at(&p, &line, 20.0).unwrap();
                assert_eq!(layout.source(), &source);
                let points = layout
                    .run_views()
                    .flat_map(|run| {
                        let origin = run.origin();
                        run.glyphs()
                            .map(|glyph| {
                                if orientation == LineOrientation::Horizontal {
                                    Point::new(
                                        origin.x + glyph.pen() + f64::from(glyph.x()),
                                        origin.y + f64::from(glyph.y()),
                                    )
                                } else {
                                    Point::new(
                                        origin.x - f64::from(glyph.y()),
                                        origin.y + glyph.pen() + f64::from(glyph.x()),
                                    )
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let base = points[1];
                let mark = points[2];
                assert_eq!(
                    (mark.x - base.x, mark.y - base.y),
                    if orientation == LineOrientation::Horizontal {
                        (3.0, -4.0)
                    } else {
                        (4.0, 3.0)
                    }
                );
            }
        }
    }
}

#[test]
fn signed_authored_terms_cancel_intrinsic_width_without_erasing_intrinsic_eligibility() {
    let bytes = mapped('\u{1361}', false);
    let mut system = super::system(&bytes, '\u{1361}', false, false);
    let source = Source::identified("A\u{1361}A", Id::from_u64(1564012), 1);
    for orientation in FRAMES {
        let control = super::prepare(&mut system, &source, orientation, 0.0);
        let natural = finalize(&control);
        for (word, positions, total) in [
            (-4.0, [0.0, 8.0, 16.0], 26.0),
            (-10.0, [0.0, 5.0, 10.0], 20.0),
        ] {
            let p = super::prepare(&mut system, &source, orientation, word);
            let line = finalize(&p);
            assert_eq!(raw(&line), raw(&natural));
            assert_eq!(line.feedback().advance, total);
            assert_eq!(paints(&line), positions);
            super::assert_placement(&p, line, &source, positions, total, false);
        }
    }
}

#[test]
fn native_spaces_allocation_remains_trailing_while_authored_spacing_uses_its_own_halves() {
    let source = Source::new("A A");
    for orientation in FRAMES {
        for zero in [false, true] {
            let native_zero = zero && orientation == LineOrientation::Vertical;
            let bytes = if native_zero {
                vertical_zero_space()
            } else {
                mapped(' ', zero)
            };
            let mut system = super::system(&bytes, ' ', zero, native_zero);
            let p = super::prepare(&mut system, &source, orientation, 4.0);
            let mut selection = request(&p);
            selection.context.orientation = orientation;
            selection.context.available_inline = None;
            selection.context.spacing = SpacingPolicy::Spaces;
            let selected = p.select(selection).unwrap();
            let slots = selected.spacing_slots().collect::<Vec<_>>();
            assert_eq!(slots.len(), 1);
            let natural = selected.finalize(SpacingAllocations::None).unwrap();
            let line = selected
                .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                    slot: slots[0].clone(),
                    amount: 6.0,
                }]))
                .unwrap();
            assert_eq!(raw(&line), raw(&natural));
            let body = if native_zero { 12.0 } else { 10.0 };
            assert_eq!(
                line.feedback().advance,
                if zero { body * 2.0 + 6.0 } else { 40.0 }
            );
            assert_eq!(
                paints(&line),
                if zero {
                    [0.0, body, body + 6.0]
                } else {
                    [0.0, 12.0, 30.0]
                }
            );
        }
    }
}

#[test]
fn an_author_forced_cross_unit_provider_group_is_preserved_inactive_and_refuses_material_word_spacing()
 {
    let bytes =
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 58], &[(0x20, 58), (0x41, 35)]);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert!(parsed.gsub().is_ok());
    assert_eq!(parsed.charmap().map(' '), Some(GlyphId::new(58)));
    let mut system = register(bytes);
    let source = Source::identified("A A", Id::from_u64(1564013), 2);
    let mut own = style(0.0);
    own.font = own.font.feature("\"ccmp\" 1");
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        let control = prepared(&mut system, &source, own.clone(), orientation, &[0, 3]);
        let natural = finalize(&control);
        assert_eq!(
            raw(&natural),
            [
                (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
                (Range::new(2, 3), 35, 0.0, 0.0, 10.0)
            ]
        );
        assert_eq!(natural.feedback().advance, 20.0);
        let mut demanded = own.clone();
        demanded.word_spacing = 4.0;
        let analysis = analyze(input(&source, demanded, &[0, 3]), orientation);
        let error = system
            .prepare(
                &analysis,
                PreparationOptions::default().with_orientation(orientation),
            )
            .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
        let Some(crate::ErrorDetail::UnsupportedSpacing {
            stage,
            operation,
            pieces,
            provider_atoms,
            ..
        }) = error.detail()
        else {
            panic!("expected actual group diagnostic: {error:?}")
        };
        assert_eq!(*stage, crate::SpacingStage::Preparation);
        assert_eq!(*operation, crate::SpacingOperation::AuthoredWord);
        assert_eq!(
            pieces.as_ref(),
            [crate::SpacingPiece::Source {
                origin: 0,
                local: Range::new(1, 2),
                aggregate: Range::new(1, 2)
            }]
        );
        assert_eq!(provider_atoms.as_ref(), std::slice::from_ref(&(0..2)));
        assert_eq!(raw(&finalize(&control)), raw(&natural));
    }
}

#[test]
fn a_local_nonzero_separator_half_must_survive_its_actual_physical_inline_coordinate() {
    let bytes = mapped('\u{1361}', false);
    let mut system = super::system(&bytes, '\u{1361}', false, false);
    let source = Source::identified("\u{1361}A", Id::from_u64(1564014), 1);
    assert_eq!(1e16_f64 + 0.5, 1e16_f64);
    for orientation in FRAMES {
        let p = prepared(&mut system, &source, style(1.0), orientation, &[0, 2]);
        let line = finalize(&p);
        assert_eq!(line.feedback().advance, 21.0);
        assert_eq!(paints(&line), [0.5, 11.0]);
        let retained = place_at(&p, &line, 20.0).unwrap();
        let glyphs = retained.glyph_runs();
        let bounds = retained.bounds();
        let error = place_at(&p, &line, 1e16).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric { phase: crate::TextNumericPhase::Placement, category: crate::TextNumericError::NonZeroLost, value, .. }) if *value == 0.5),
            "{error:?}"
        );
        assert_eq!(retained.glyph_runs(), glyphs);
        assert_eq!(retained.bounds(), bounds);
    }
}

#[test]
fn a_required_half_cannot_disappear_in_the_finalized_local_provider_pen() {
    let bytes = mapped('\u{1361}', false);
    let mut system = super::system(&bytes, '\u{1361}', false, false);
    let source = Source::new("A\u{1361}");
    let size = 18_014_398_509_481_984.0_f32; // exactly2^54
    let advance = f64::from(size);
    assert_eq!(advance + 2.0, advance);
    assert!(advance + 4.0 > advance);
    let own = Style {
        size,
        line_height: LineHeight::Absolute(advance),
        ..style(0.0)
    };
    let control = prepared(
        &mut system,
        &source,
        own.clone(),
        LineOrientation::Horizontal,
        &[0, 2],
    );
    let natural = finalize(&control);
    assert_eq!(
        raw(&natural).iter().map(|row| row.4).collect::<Vec<_>>(),
        [size, size]
    );
    assert_eq!(paints(&natural), [0.0, advance]);
    let demanded = prepared(
        &mut system,
        &source,
        Style {
            word_spacing: 4.0,
            ..own
        },
        LineOrientation::Horizontal,
        &[0, 2],
    );
    let mut selection = request(&demanded);
    selection.context.available_inline = None;
    selection.context.spacing = SpacingPolicy::None;
    let error = demanded
        .select(selection)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
    assert!(
        matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric { phase: crate::TextNumericPhase::Finalization, category: crate::TextNumericError::NonZeroLost, value, .. }) if *value == 2.0),
        "{error:?}"
    );
    assert_eq!(paints(&finalize(&control)), [0.0, advance]);
}

#[test]
fn hypothetical_block_space_uses_the_same_intrinsic_zero_gate_without_inventing_a_source_space() {
    let source = Source::identified("A\tA", Id::from_u64(1564015), 1);
    for orientation in FRAMES {
        for zero in [false, true] {
            let native_zero = zero && orientation == LineOrientation::Vertical;
            let mapping = crate::shaping_test_fonts::range_ligature_ahem(
                &[82, 82],
                &[(0x20, 58), (0x30, 18), (0x41, 35)],
            );
            let bytes = if native_zero {
                sfnt::replace_tables(
                    &vertical::zero_signed(),
                    vec![(*b"cmap", sfnt::table(&mapping, *b"cmap").to_vec())],
                )
            } else if zero {
                let mut hmtx = sfnt::table(&mapping, *b"hmtx").to_vec();
                hmtx[58 * 4..58 * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
                sfnt::replace_tables(&mapping, vec![(*b"hmtx", hmtx)])
            } else {
                mapping
            };
            let parsed = skrifa::FontRef::new(&bytes).unwrap();
            assert_eq!(parsed.charmap().map('0'), Some(GlyphId::new(18)));
            assert_eq!(
                parsed
                    .glyph_metrics(FontSize::unscaled(), LocationRef::default())
                    .advance_width(GlyphId::new(18)),
                Some(1000.0)
            );
            let mut system = super::system(&bytes, ' ', zero, native_zero);
            let own = Style {
                tab_size: crate::TabSize::spaces(4.0).unwrap(),
                ..style(4.0)
            };
            let mut value = input(&source, own.clone(), &[0, 3]);
            value.tab_reference = Some(
                crate::ParagraphTabReference::new(own, ParagraphBaseDirection::LeftToRight)
                    .unwrap(),
            );
            let p = system
                .prepare(
                    &analyze(value, orientation),
                    PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops)
                        .unwrap()
                        .with_orientation(orientation),
                )
                .unwrap();
            let mut selection = request(&p);
            selection.context.orientation = orientation;
            selection.context.available_inline = None;
            selection.context.spacing = SpacingPolicy::None;
            selection.context.tabs = TabContext::CssStops { origin: Some(0.0) };
            let line = p
                .select(selection)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            // Positive block SPACE10 plus authored4 gives a56px grid and
            // body A at0/56. Intrinsic0 SPACE suppresses4: period0, no tab
            // shift. Native vmtx0 is independent of its body A12 metrics.
            let body = if native_zero { 12.0 } else { 10.0 };
            assert_eq!(
                line.feedback().advance,
                if zero { body * 2.0 } else { 66.0 }
            );
            assert_eq!(
                paints(&line),
                if zero {
                    vec![0.0, body]
                } else {
                    vec![0.0, 56.0]
                }
            );
            assert_eq!(
                raw(&line).iter().map(|row| row.0).collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(2, 3)]
            );
            assert_eq!(
                p.analyzed_source().source().text().code_units(),
                [65, 9, 65]
            );
        }
    }
}

fn indication_prefix(prepared: &PreparedText) -> FinalizedLine {
    let cut = prepared
        .hyphenation_cut_views()
        .find(|cut| cut.boundary().view().source_before().get() == 6)
        .unwrap();
    assert_eq!(
        cut.counts(),
        Some(crate::HyphenationCounts {
            total: 10,
            before: 5,
            after: 5
        })
    );
    let offer = cut.opportunity().unwrap();
    let alternative = offer.view().alternatives().next().unwrap().handle();
    assert_eq!(alternative.view().text().code_units(), [0x1361]);
    assert_eq!(alternative.view().source_range(), Some(Range::new(5, 6)));
    let mut selection = request(prepared);
    selection.context.orientation = prepared.orientation();
    selection.context.available_inline = None;
    selection.context.spacing = SpacingPolicy::None;
    selection.context.end = LineEnd::Soft;
    selection.end = offer.view().boundary();
    selection.participants = prepared
        .participants()
        .take(selection.end.view().participant_offset())
        .collect();
    selection.termination = crate::PreparedTermination::Ordinary(offer);
    selection.alternative = Some(alternative);
    prepared
        .select(selection)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

#[test]
fn an_actual_selected_indication_gets_authored_spacing_from_its_own_complete_separator() {
    let bytes = mapped('\u{1361}', false);
    let mut system = super::system(&bytes, '\u{1361}', false, false);
    let source = Source::identified("AAAAA\u{ad}AAAAA", Id::from_u64(1564016), 2);
    for orientation in FRAMES {
        let own = Style {
            hyphens: crate::Hyphens::Manual,
            hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("\u{1361}")),
            ..style(0.0)
        };
        let control = prepared(&mut system, &source, own.clone(), orientation, &[0, 11]);
        let natural = indication_prefix(&control);
        assert_eq!(natural.feedback().advance, 60.0);
        assert_eq!(paints(&natural), [0.0, 10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(raw(&natural).last().unwrap().0, Range::new(6, 6));
        assert_eq!(raw(&natural).last().unwrap().4, 10.0);
        let p = prepared(
            &mut system,
            &source,
            Style {
                word_spacing: 4.0,
                ..own
            },
            orientation,
            &[0, 11],
        );
        let line = indication_prefix(&p);
        assert_eq!(raw(&line), raw(&natural));
        assert_eq!(line.feedback().advance, 64.0);
        assert_eq!(paints(&line), [0.0, 10.0, 20.0, 30.0, 40.0, 52.0]);
        assert_eq!(p.analyzed_source().source(), &source);
    }
}

#[test]
fn contained_groups_from_real_fallback_fonts_move_together_without_a_cross_font_attachment_claim() {
    const PHRASE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf"
    ));
    let primary = mapped('\u{1361}', false);
    let mark_mapping = crate::shaping_test_fonts::range_ligature_ahem(&[82, 82], &[(0x301, 100)]);
    let mut hmtx = sfnt::table(PHRASE, *b"hmtx").to_vec();
    hmtx[100 * 4..100 * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
    let fallback = sfnt::replace_tables(
        PHRASE,
        vec![
            (*b"cmap", sfnt::table(&mark_mapping, *b"cmap").to_vec()),
            (*b"hmtx", hmtx),
        ],
    );
    let first = skrifa::FontRef::new(&primary).unwrap();
    let second = skrifa::FontRef::new(&fallback).unwrap();
    assert_eq!(first.charmap().map('\u{301}'), None);
    assert_eq!(second.charmap().map('\u{301}'), Some(GlyphId::new(100)));
    assert_eq!(
        second
            .glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(100)),
        Some(0.0)
    );
    let mut system = register(primary.clone());
    system.register_fonts(fallback.clone()).unwrap();
    let mut source = Source::identified("A\u{1361}\u{301}A", Id::from_u64(1564017), 1);
    // This is a genuine authored font-request difference on the mark. A
    // uniform EGC query with two equally partial faces does not union them.
    // The mark's own query now falls through Ahem0/1 to Phrase Ahem1/1.
    source.span(
        Range::new(2, 3),
        Style {
            font: Font::new().family("Ahem").family("Surgeist Phrase Ahem"),
            ..style(0.0)
        },
    );
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        let own = style(0.0);
        let control = prepared(&mut system, &source, own.clone(), orientation, &[0, 2, 4]);
        assert_eq!(
            control.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
        );
        let natural = finalize(&control);
        let actual_mark_run = natural
            .run_views()
            .find(|run| run.glyphs().any(|glyph| glyph.id() == 100))
            .expect("the mark's genuine font request must select its covering fallback glyph");
        assert_eq!(
            actual_mark_run.face().bytes(),
            fallback,
            "the raw control must retain the actual covering Phrase Ahem resource"
        );
        assert_eq!(
            raw(&natural)
                .iter()
                .map(|glyph| (glyph.1, glyph.4))
                .collect::<Vec<_>>(),
            [(35, 10.0), (58, 10.0), (100, 0.0), (35, 10.0)]
        );
        assert_eq!(natural.feedback().advance, 30.0);
        let original_paints = paints(&natural);
        let p = prepared(
            &mut system,
            &source,
            Style {
                word_spacing: 4.0,
                ..own
            },
            orientation,
            &[0, 2, 4],
        );
        let line = finalize(&p);
        assert_eq!(raw(&line), raw(&natural));
        assert_eq!(line.feedback().advance, 34.0);
        assert_eq!(
            paints(&line),
            [
                original_paints[0],
                original_paints[1] + 2.0,
                original_paints[2] + 2.0,
                original_paints[3] + 4.0
            ]
        );
        let fallback_run = line
            .run_views()
            .find(|run| run.glyphs().any(|glyph| glyph.id() == 100))
            .unwrap();
        assert_eq!(fallback_run.face().bytes(), fallback);
        assert!(
            line.run_views()
                .filter(|run| run
                    .glyphs()
                    .any(|glyph| glyph.id() == 35 || glyph.id() == 58))
                .all(|run| run.face().bytes() == primary)
        );
        let layout = place_at(&p, &line, 20.0).unwrap();
        assert_eq!(layout.source(), &source);
    }
}
